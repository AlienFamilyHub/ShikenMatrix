//! Native compositor IPC and X11 EWMH window discovery.

#[path = "icons.rs"]
mod icons;

use crate::platform::WindowInfo;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

const MAX_RESPONSE: usize = 16 * 1024 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, PartialEq)]
enum Backend {
    Niri(PathBuf),
    Sway(PathBuf),
    Hyprland(PathBuf),
    X11,
}

fn detect_backend(mut env: impl FnMut(&str) -> Option<String>) -> Result<Backend, String> {
    let session = env("XDG_SESSION_TYPE").unwrap_or_default();
    let wayland = session.eq_ignore_ascii_case("wayland")
        || env("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
    if let Some(socket) = env("NIRI_SOCKET").filter(|value| !value.is_empty()) {
        return Ok(Backend::Niri(socket.into()));
    }
    if let Some(socket) = env("SWAYSOCK").filter(|value| !value.is_empty()) {
        return Ok(Backend::Sway(socket.into()));
    }
    if let Some(signature) = env("HYPRLAND_INSTANCE_SIGNATURE").filter(|value| !value.is_empty()) {
        if signature.contains(['/', '\\', '\0']) || signature == "." || signature == ".." {
            return Err("Invalid HYPRLAND_INSTANCE_SIGNATURE".into());
        }
        let runtime = env("XDG_RUNTIME_DIR").ok_or("Hyprland requires XDG_RUNTIME_DIR")?;
        return Ok(Backend::Hyprland(
            Path::new(&runtime)
                .join("hypr")
                .join(signature)
                .join(".socket.sock"),
        ));
    }
    if wayland {
        return Err(format!(
            "Window monitoring is unavailable for this Wayland compositor ({}). Run under niri, Sway, or Hyprland with its IPC socket environment exported; XWayland cannot enumerate native Wayland windows.",
            env("XDG_CURRENT_DESKTOP").unwrap_or_else(|| "unknown".into())
        ));
    }
    if env("DISPLAY").is_some_and(|value| !value.is_empty()) {
        return Ok(Backend::X11);
    }
    Err(
        "No graphical window backend: export the compositor IPC environment or an X11 DISPLAY"
            .into(),
    )
}

fn backend() -> Result<Backend, String> {
    detect_backend(|name| std::env::var(name).ok())
}

#[derive(Debug)]
struct NativeWindow {
    title: String,
    app_id: Option<String>,
    pid: i32,
    focused: bool,
}

fn native_window(value: &Value, focus_key: &str) -> Result<NativeWindow, String> {
    if !value.is_object() {
        return Err("Invalid compositor window object".into());
    }
    Ok(NativeWindow {
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        app_id: value
            .get("app_id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_owned),
        pid: value
            .get("pid")
            .and_then(Value::as_i64)
            .and_then(|pid| i32::try_from(pid).ok())
            .filter(|pid| *pid > 0)
            .unwrap_or(0),
        focused: value
            .get(focus_key)
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn parse_niri(value: &Value, response: &str) -> Result<Vec<NativeWindow>, String> {
    let ok = value
        .get("Ok")
        .ok_or_else(|| format!("niri IPC rejected {response}: {value}"))?;
    let payload = ok
        .get(response)
        .ok_or_else(|| format!("Unexpected niri {response} response"))?;
    if response == "FocusedWindow" {
        return if payload.is_null() {
            Ok(Vec::new())
        } else {
            let mut window = native_window(payload, "is_focused")?;
            window.focused = true;
            Ok(vec![window])
        };
    }
    payload
        .as_array()
        .ok_or("Invalid niri Windows response")?
        .iter()
        .map(|value| native_window(value, "is_focused"))
        .collect()
}

fn parse_sway(tree: &Value) -> Result<Vec<NativeWindow>, String> {
    fn visit(node: &Value, windows: &mut Vec<NativeWindow>, depth: usize) -> Result<(), String> {
        if !node.is_object() || depth > 128 {
            return Err("Invalid or excessively nested Sway tree".into());
        }
        let app_id = node
            .get("app_id")
            .and_then(Value::as_str)
            .or_else(|| node.get("window_properties")?.get("class")?.as_str());
        let is_window = node.get("type").and_then(Value::as_str) == Some("con")
            && (app_id.is_some()
                || node.get("shell").and_then(Value::as_str).is_some()
                || node
                    .get("window")
                    .and_then(Value::as_u64)
                    .is_some_and(|id| id != 0));
        if is_window {
            let mut window = native_window(node, "focused")?;
            window.title = node
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            window.app_id = app_id.filter(|s| !s.is_empty()).map(str::to_owned);
            windows.push(window);
        }
        for key in ["nodes", "floating_nodes"] {
            if let Some(children) = node.get(key) {
                for child in children.as_array().ok_or("Invalid Sway tree children")? {
                    visit(child, windows, depth + 1)?;
                }
            }
        }
        Ok(())
    }
    let mut windows = Vec::new();
    visit(tree, &mut windows, 0)?;
    Ok(windows)
}

fn parse_hyprland(clients: &Value, active: Option<&str>) -> Result<Vec<NativeWindow>, String> {
    let mut windows = Vec::new();
    for client in clients
        .as_array()
        .ok_or("Invalid Hyprland clients response")?
    {
        if !client.is_object() {
            return Err("Invalid Hyprland client object".into());
        }
        if client.get("mapped").and_then(Value::as_bool) == Some(false)
            || client.get("hidden").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        let mut window = native_window(client, "focused")?;
        window.app_id = client
            .get("class")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        window.focused =
            active.is_some() && client.get("address").and_then(Value::as_str) == active;
        windows.push(window);
    }
    Ok(windows)
}

fn socket(path: &Path) -> Result<UnixStream, String> {
    let stream = UnixStream::connect(path).map_err(|error| {
        format!(
            "Cannot connect to compositor socket {}: {error}",
            path.display()
        )
    })?;
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(IPC_TIMEOUT))
        .map_err(|error| error.to_string())?;
    Ok(stream)
}

fn json_response(bytes: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("Malformed compositor JSON: {error}"))
}

fn niri_request(path: &Path, request: &str) -> Result<Value, String> {
    let mut stream = socket(path)?;
    let mut command = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    command.push(b'\n');
    stream
        .write_all(&command)
        .map_err(|error| format!("niri IPC write: {error}"))?;
    let mut bytes = Vec::new();
    BufReader::new(stream)
        .take((MAX_RESPONSE + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| format!("niri IPC read: {error}"))?;
    if bytes.len() > MAX_RESPONSE || bytes.last() != Some(&b'\n') {
        return Err("Truncated or oversized niri IPC response".into());
    }
    json_response(&bytes)
}

fn sway_payload_len(header: &[u8], kind: u32) -> Result<usize, String> {
    if header.len() != 14
        || &header[..6] != b"i3-ipc"
        || u32::from_le_bytes(
            header[10..14]
                .try_into()
                .map_err(|_| "Invalid Sway header")?,
        ) != kind
    {
        return Err("Invalid Sway IPC frame".into());
    }
    let length = u32::from_le_bytes(
        header[6..10]
            .try_into()
            .map_err(|_| "Invalid Sway length")?,
    ) as usize;
    if length > MAX_RESPONSE {
        return Err("Oversized Sway IPC response".into());
    }
    Ok(length)
}

fn sway_request(path: &Path) -> Result<Value, String> {
    let mut stream = socket(path)?;
    let mut request = b"i3-ipc".to_vec();
    request.extend_from_slice(&0u32.to_le_bytes());
    request.extend_from_slice(&4u32.to_le_bytes()); // GET_TREE
    stream
        .write_all(&request)
        .map_err(|error| format!("Sway IPC write: {error}"))?;
    let mut header = [0u8; 14];
    stream
        .read_exact(&mut header)
        .map_err(|error| format!("Sway IPC header: {error}"))?;
    let mut bytes = vec![0; sway_payload_len(&header, 4)?];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| format!("Sway IPC payload: {error}"))?;
    json_response(&bytes)
}

fn hyprland_request(path: &Path, request: &[u8]) -> Result<Value, String> {
    let mut stream = socket(path)?;
    stream
        .write_all(request)
        .map_err(|error| format!("Hyprland IPC write: {error}"))?;
    let mut bytes = Vec::new();
    stream
        .take((MAX_RESPONSE + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Hyprland IPC read: {error}"))?;
    if bytes.len() > MAX_RESPONSE {
        return Err("Oversized Hyprland IPC response".into());
    }
    json_response(&bytes)
}

impl Backend {
    fn windows(&self, focused_only: bool) -> Result<Vec<NativeWindow>, String> {
        match self {
            Self::Niri(path) => {
                let request = if focused_only {
                    "FocusedWindow"
                } else {
                    "Windows"
                };
                parse_niri(&niri_request(path, request)?, request)
            }
            Self::Sway(path) => parse_sway(&sway_request(path)?),
            Self::Hyprland(path) => {
                let clients = hyprland_request(path, b"j/clients")?;
                let active = if focused_only {
                    Some(hyprland_request(path, b"j/activewindow")?)
                } else {
                    None
                };
                parse_hyprland(
                    &clients,
                    active
                        .as_ref()
                        .and_then(|value| value.get("address"))
                        .and_then(Value::as_str),
                )
            }
            Self::X11 => Err("Internal backend mismatch".into()),
        }
    }
}

fn process_identity(pid: i32) -> (String, Option<String>) {
    if pid <= 0 {
        return (String::new(), None);
    }
    let proc = PathBuf::from("/proc").join(pid.to_string());
    let executable = std::fs::read_link(proc.join("exe")).ok();
    let name = std::fs::File::open(proc.join("comm"))
        .ok()
        .and_then(|file| {
            let mut bytes = Vec::new();
            file.take(4096).read_to_end(&mut bytes).ok()?;
            String::from_utf8(bytes)
                .ok()
                .map(|value| value.trim_end().to_owned())
        })
        .filter(|value| !value.is_empty())
        .or_else(|| {
            executable
                .as_ref()?
                .file_name()?
                .to_str()
                .map(str::to_owned)
        })
        .unwrap_or_default();
    (
        name,
        executable.map(|path| path.to_string_lossy().into_owned()),
    )
}

fn window_info(window: NativeWindow) -> WindowInfo {
    window_info_with_icon(window, None)
}

fn window_info_with_icon(window: NativeWindow, icon: Option<Vec<u8>>) -> WindowInfo {
    let (mut process_name, executable) = process_identity(window.pid);
    let app_id = window.app_id.or(executable);
    if process_name.is_empty() {
        process_name = app_id
            .as_deref()
            .and_then(|app| Path::new(app).file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
    }
    let icon_data = icon.or_else(|| icons::desktop_icon(app_id.as_deref(), &process_name));
    WindowInfo {
        title: window.title,
        icon_data,
        process_name,
        pid: window.pid,
        app_id,
    }
}

fn decode_x11_text(bytes: &[u8], utf8: bool) -> Option<String> {
    let text = if utf8 {
        std::str::from_utf8(bytes).ok()?.to_owned()
    } else {
        bytes.iter().map(|byte| char::from(*byte)).collect()
    };
    Some(text.trim_end_matches('\0').to_owned())
}

struct X11 {
    connection: RustConnection,
    root: Window,
}

impl X11 {
    fn connect() -> Result<Self, String> {
        let (connection, screen) = x11rb::connect(None)
            .map_err(|error| format!("Cannot connect to X11 DISPLAY: {error}"))?;
        let root = connection.setup().roots[screen].root;
        Ok(Self { connection, root })
    }

    fn atom(&self, name: &str) -> Result<Atom, String> {
        self.connection
            .intern_atom(false, name.as_bytes())
            .map_err(|error| error.to_string())?
            .reply()
            .map(|reply| reply.atom)
            .map_err(|error| error.to_string())
    }

    fn property(
        &self,
        window: Window,
        name: &str,
        kind: Atom,
        limit: u32,
    ) -> Result<x11rb::protocol::xproto::GetPropertyReply, String> {
        self.connection
            .get_property(false, window, self.atom(name)?, kind, 0, limit)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())
    }

    fn clients(&self) -> Result<Vec<Window>, String> {
        for name in ["_NET_CLIENT_LIST_STACKING", "_NET_CLIENT_LIST"] {
            let reply = self.property(self.root, name, AtomEnum::WINDOW.into(), 32768)?;
            if reply.type_ != u32::from(AtomEnum::NONE) {
                if reply.bytes_after != 0 {
                    return Err("X11 client list exceeds the supported size".into());
                }
                return reply
                    .value32()
                    .map(Iterator::collect)
                    .ok_or_else(|| "Malformed X11 client list".into());
            }
        }
        Err("X11 window manager does not expose EWMH _NET_CLIENT_LIST; use an EWMH-compatible window manager".into())
    }

    fn active(&self) -> Result<Window, String> {
        self.property(self.root, "_NET_ACTIVE_WINDOW", AtomEnum::WINDOW.into(), 1)?
            .value32()
            .and_then(|mut values| values.next())
            .filter(|window| *window != 0)
            .ok_or_else(|| "No focused X11 window".into())
    }

    fn text(&self, window: Window, name: &str, kind: Atom) -> Option<String> {
        let reply = self.property(window, name, kind, 65536).ok()?;
        if reply.format != 8 || reply.value.is_empty() || reply.bytes_after != 0 {
            return None;
        }
        decode_x11_text(&reply.value, name == "_NET_WM_NAME")
    }

    fn info(&self, window: Window) -> Result<Option<WindowInfo>, String> {
        match self
            .connection
            .get_window_attributes(window)
            .map_err(|error| error.to_string())?
            .reply()
        {
            Ok(_) => {}
            Err(x11rb::errors::ReplyError::X11Error(error))
                if error.error_kind == x11rb::protocol::ErrorKind::Window =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error.to_string()),
        }
        let title = self
            .text(window, "_NET_WM_NAME", self.atom("UTF8_STRING")?)
            .or_else(|| self.text(window, "WM_NAME", AtomEnum::STRING.into()))
            .unwrap_or_default();
        let pid = self
            .property(window, "_NET_WM_PID", AtomEnum::CARDINAL.into(), 1)
            .ok()
            .and_then(|reply| reply.value32().and_then(|mut values| values.next()))
            .and_then(|pid| i32::try_from(pid).ok())
            .filter(|pid| *pid > 0)
            .unwrap_or(0);
        let class = self
            .text(window, "WM_CLASS", AtomEnum::STRING.into())
            .and_then(|value| {
                value
                    .split('\0')
                    .rev()
                    .find(|part| !part.is_empty())
                    .map(str::to_owned)
            });
        let icon = self
            .property(
                window,
                "_NET_WM_ICON",
                AtomEnum::CARDINAL.into(),
                (MAX_RESPONSE / 4) as u32,
            )
            .ok()
            .filter(|reply| reply.bytes_after == 0)
            .and_then(|reply| icons::ewmh_icon(&reply.value32()?.collect::<Vec<_>>()));
        Ok(Some(window_info_with_icon(
            NativeWindow {
                title,
                app_id: class,
                pid,
                focused: false,
            },
            icon,
        )))
    }
}

fn permissions_for_backend(backend: &Backend) -> Result<bool, String> {
    match backend {
        Backend::X11 => {
            X11::connect()?.clients()?;
        }
        backend => {
            backend.windows(false)?;
        }
    }
    Ok(true)
}

/// Linux needs no permission prompt; validate access to the actual native backend.
pub fn request_permissions() -> Result<bool, String> {
    permissions_for_backend(&backend()?)
}

pub fn check_permissions() -> bool {
    request_permissions().unwrap_or(false)
}

pub fn get_frontmost_window() -> Result<WindowInfo, String> {
    let backend = backend()?;
    match backend {
        Backend::X11 => {
            let x11 = X11::connect()?;
            x11.info(x11.active()?)?
                .ok_or_else(|| "Focused X11 window closed during inspection".into())
        }
        backend => backend
            .windows(true)?
            .into_iter()
            .find(|window| window.focused)
            .map(window_info)
            .ok_or_else(|| "No focused window in the compositor".into()),
    }
}

pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    let backend = backend()?;
    match backend {
        Backend::X11 => {
            let x11 = X11::connect()?;
            // Windows can close between the list query and property queries.
            let mut windows = Vec::new();
            for id in x11.clients()? {
                if let Some(window) = x11.info(id)? {
                    windows.push(window);
                }
            }
            Ok(windows)
        }
        backend => Ok(backend
            .windows(false)?
            .into_iter()
            .map(window_info)
            .collect()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn legacy_x11_titles_decode_latin1_and_invalid_utf8_falls_back() {
        assert_eq!(
            decode_x11_text(b"caf\xe9\0", false).as_deref(),
            Some("café")
        );
        assert!(decode_x11_text(b"caf\xe9", true).is_none());
        assert_eq!(
            decode_x11_text("日本語".as_bytes(), true).as_deref(),
            Some("日本語")
        );
    }

    use super::*;
    use serde_json::json;

    fn route(vars: &[(&str, &str)]) -> Result<Backend, String> {
        detect_backend(|key| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).into())
        })
    }

    #[test]
    fn wayland_never_falls_back_to_xwayland() {
        assert!(route(&[("XDG_SESSION_TYPE", "wayland"), ("DISPLAY", ":0")]).is_err());
        assert!(matches!(
            route(&[
                ("WAYLAND_DISPLAY", "wayland-0"),
                ("DISPLAY", ":0"),
                ("NIRI_SOCKET", "/tmp/niri")
            ]),
            Ok(Backend::Niri(_))
        ));
        assert!(matches!(route(&[("DISPLAY", ":0")]), Ok(Backend::X11)));
    }

    #[test]
    fn native_socket_routes_do_not_require_display() {
        assert!(matches!(
            route(&[("SWAYSOCK", "/tmp/sway")]),
            Ok(Backend::Sway(_))
        ));
        assert!(matches!(
            route(&[
                ("HYPRLAND_INSTANCE_SIGNATURE", "abc"),
                ("XDG_RUNTIME_DIR", "/run/user/1000")
            ]),
            Ok(Backend::Hyprland(_))
        ));
    }

    #[test]
    fn niri_preserves_native_identity_and_focus() {
        let windows = parse_niri(
            &json!({"Ok":{"Windows":[
                {"id":1,"title":"Editor","app_id":"org.editor.App","pid":42,"is_focused":true},
                {"id":2,"title":"Browser","app_id":"browser","pid":null,"is_focused":false}
            ]}}),
            "Windows",
        )
        .unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].app_id.as_deref(), Some("org.editor.App"));
        assert_eq!(windows[0].pid, 42);
        assert!(windows[0].focused);
        assert_eq!(windows[1].pid, 0);
        assert!(
            parse_niri(&json!({"Ok":{"FocusedWindow":null}}), "FocusedWindow")
                .unwrap()
                .is_empty()
        );
        assert!(parse_niri(&json!({"Err":"denied"}), "Windows").is_err());
        assert!(parse_niri(&json!({"Ok":{"Windows":{}}}), "Windows").is_err());
    }

    #[test]
    fn sway_filters_containers_and_includes_floating_windows() {
        let tree = json!({"type":"root","nodes":[{"type":"workspace","name":"1","nodes":[
            {"type":"con","name":"Native","app_id":"editor","pid":42,"focused":false,"nodes":[],"floating_nodes":[]},
            {"type":"con","name":"Unnamed application","app_id":null,"shell":"xdg_shell","pid":43,"nodes":[],"floating_nodes":[]},
            {"type":"con","name":"layout","nodes":[],"floating_nodes":[]}
        ],"floating_nodes":[{"type":"floating_con","nodes":[
            {"type":"con","name":"Legacy","window":100,"window_properties":{"class":"LegacyApp"},"focused":true}
        ]}]}]});
        let windows = parse_sway(&tree).unwrap();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[1].title, "Unnamed application");
        assert_eq!(windows[1].pid, 43);
        assert_eq!(windows[2].app_id.as_deref(), Some("LegacyApp"));
        assert!(windows[2].focused);
        assert!(parse_sway(&json!([])).is_err());
    }

    #[test]
    fn hyprland_excludes_unmapped_clients_and_matches_active_address() {
        let windows = parse_hyprland(&json!([
            {"address":"0x1","mapped":true,"hidden":false,"title":"Visible","class":"editor","pid":42},
            {"address":"0x2","mapped":false,"title":"Closed","class":"editor"},
            {"address":"0x3","mapped":true,"hidden":true,"title":"Hidden","class":"editor"}
        ]), Some("0x1")).unwrap();
        assert_eq!(windows.len(), 1);
        assert!(windows[0].focused);
        assert_eq!(windows[0].title, "Visible");
        assert!(parse_hyprland(&json!({}), None).is_err());
    }

    #[test]
    fn sway_frames_reject_wrong_magic_kind_and_oversize() {
        let mut frame = b"i3-ipc".to_vec();
        frame.extend_from_slice(&2u32.to_le_bytes());
        frame.extend_from_slice(&4u32.to_le_bytes());
        assert_eq!(sway_payload_len(&frame, 4).unwrap(), 2);
        assert!(sway_payload_len(&frame, 0).is_err());
        frame[0] = b'x';
        assert!(sway_payload_len(&frame, 4).is_err());
        frame[0] = b'i';
        frame[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(sway_payload_len(&frame, 4).is_err());
    }

    #[test]
    fn available_compositor_with_no_windows_still_has_permission() {
        let (path, server) = niri_server(b"{\"Ok\":{\"Windows\":[]}}\n");
        assert!(permissions_for_backend(&Backend::Niri(path.clone())).unwrap());
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn niri_rejects_unterminated_json_frames() {
        let (path, server) = niri_server(b"{\"Ok\":{\"Windows\":[]}}");
        assert!(niri_request(&path, "Windows").is_err());
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }

    fn niri_server(response: &'static [u8]) -> (PathBuf, std::thread::JoinHandle<()>) {
        use std::os::unix::net::UnixListener;
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "shikenmatrix-window-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut request)
                .unwrap();
            assert_eq!(request, "\"Windows\"\n");
            stream.write_all(response).unwrap();
        });
        (path, server)
    }
}
