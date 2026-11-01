//! Bounded desktop-file and EWMH icon loading. Unsupported formats are omitted.

use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

const MAX_ICON_FILE: u64 = 8 * 1024 * 1024;
const MAX_ICON_SIDE: u32 = 1024;

fn png(image: image::DynamicImage) -> Option<Vec<u8>> {
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png).ok()?;
    Some(output.into_inner())
}

pub(super) fn ewmh_icon(values: &[u32]) -> Option<Vec<u8>> {
    let mut offset = 0usize;
    let mut best: Option<(u32, u32, &[u32])> = None;
    while offset < values.len() {
        let width = *values.get(offset)?;
        let height = *values.get(offset + 1)?;
        let count = (width as usize).checked_mul(height as usize)?;
        let start = offset.checked_add(2)?;
        let end = start.checked_add(count)?;
        let pixels = values.get(start..end)?;
        if width == 0 || height == 0 {
            return None;
        }
        if width <= MAX_ICON_SIDE
            && height <= MAX_ICON_SIDE
            && best
                .as_ref()
                .is_none_or(|(w, h, _)| count > (*w as usize * *h as usize))
        {
            best = Some((width, height, pixels));
        }
        offset = end;
    }
    let (width, height, pixels) = best?;
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for argb in pixels {
        rgba.extend_from_slice(&[
            (argb >> 16) as u8,
            (argb >> 8) as u8,
            *argb as u8,
            (argb >> 24) as u8,
        ]);
    }
    png(image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
        width, height, rgba,
    )?))
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', '\0'])
        && name.len() <= 255
}

fn desktop_icon_name(text: &str) -> Option<String> {
    let mut main = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            main = line == "[Desktop Entry]";
        } else if main {
            if let Some(value) = line.strip_prefix("Icon=") {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
}

fn read_bounded(path: &Path, limit: u64) -> Option<Vec<u8>> {
    let file = fs::File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() || file.metadata().ok()?.len() > limit {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= limit).then_some(bytes)
}

fn load_image(path: &Path) -> Option<Vec<u8>> {
    let bytes = read_bounded(path, MAX_ICON_FILE)?;
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_ICON_SIDE);
    limits.max_image_height = Some(MAX_ICON_SIDE);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    png(reader.decode().ok()?)
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("XDG_DATA_HOME").filter(|p| !p.is_empty()) {
        dirs.push(PathBuf::from(home));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share"));
    }
    let system = std::env::var_os("XDG_DATA_DIRS")
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs.extend(
        std::env::split_paths(&system)
            .filter(|p| p.is_absolute())
            .take(32),
    );
    dirs.retain(|p| p.is_absolute());
    dirs
}

fn desktop_entry_icon(dirs: &[PathBuf], app: &str) -> Option<String> {
    let app = app.strip_suffix(".desktop").unwrap_or(app);
    if !safe_name(app) {
        return None;
    }
    let filename = format!("{app}.desktop");
    for dir in dirs {
        let applications = dir.join("applications");
        let exact = applications.join(&filename);
        let path = if exact.is_file() {
            exact
        } else {
            let Some(entry) = fs::read_dir(&applications).ok().and_then(|entries| {
                entries.take(4096).filter_map(Result::ok).find(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(&filename))
                })
            }) else {
                continue;
            };
            entry.path()
        };
        if let Some(icon) = read_bounded(&path, 256 * 1024)
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| desktop_icon_name(&text))
        {
            return Some(icon);
        }
    }
    None
}

fn named_icon(dirs: &[PathBuf], name: &str) -> Option<Vec<u8>> {
    if !safe_name(name) {
        return None;
    }
    let filenames = if Path::new(name).extension().is_some() {
        vec![name.to_owned()]
    } else {
        vec![
            format!("{name}.png"),
            format!("{name}.xpm"),
            format!("{name}.svg"),
        ]
    };
    for dir in dirs {
        let root = dir.join("icons");
        let mut themes = vec![root.join("hicolor"), root.join("Adwaita")];
        if let Ok(entries) = fs::read_dir(&root) {
            themes.extend(
                entries
                    .take(32)
                    .filter_map(Result::ok)
                    .map(|entry| entry.path()),
            );
        }
        for theme in themes {
            for size in [
                "256x256", "128x128", "64x64", "48x48", "32x32", "scalable", "symbolic",
            ] {
                for category in ["apps", "applications"] {
                    for filename in &filenames {
                        if let Some(icon) =
                            load_image(&theme.join(size).join(category).join(filename))
                        {
                            return Some(icon);
                        }
                    }
                }
            }
        }
        for filename in &filenames {
            if let Some(icon) = load_image(&dir.join("pixmaps").join(filename))
                .or_else(|| load_image(&root.join(filename)))
            {
                return Some(icon);
            }
        }
    }
    None
}

pub(super) fn desktop_icon(app: Option<&str>, process_name: &str) -> Option<Vec<u8>> {
    let dirs = data_dirs();
    for candidate in app.into_iter().chain(std::iter::once(process_name)) {
        let candidate = Path::new(candidate).file_name()?.to_str()?;
        if let Some(name) = desktop_entry_icon(&dirs, candidate) {
            let path = Path::new(&name);
            if path.is_absolute() {
                if let Some(icon) = load_image(path) {
                    return Some(icon);
                }
            } else if let Some(icon) = named_icon(&dirs, &name) {
                return Some(icon);
            }
        }
        if let Some(icon) = named_icon(&dirs, candidate) {
            return Some(icon);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argb_icon_is_encoded_as_rgba_png() {
        let bytes = ewmh_icon(&[1, 1, 0x80402010]).unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_eq!(decoded.dimensions(), (1, 1));
        assert_eq!(decoded.get_pixel(0, 0).0, [0x40, 0x20, 0x10, 0x80]);
    }

    #[test]
    fn malformed_icon_dimensions_and_truncation_are_rejected() {
        assert!(ewmh_icon(&[0, 1]).is_none());
        assert!(ewmh_icon(&[u32::MAX, u32::MAX]).is_none());
        assert!(ewmh_icon(&[2, 2, 0]).is_none());
        assert!(ewmh_icon(&[1]).is_none());
    }

    #[test]
    fn desktop_icon_only_reads_main_desktop_entry() {
        let text =
            "[Desktop Entry]\nName=Editor\nIcon=org.editor.App\n[Desktop Action New]\nIcon=wrong\n";
        assert_eq!(desktop_icon_name(text).as_deref(), Some("org.editor.App"));
        assert!(desktop_icon_name("[Other]\nIcon=wrong\n").is_none());
        assert!(!safe_name("../../etc/passwd"));
        assert!(!safe_name("."));
        assert!(safe_name("org.editor.App"));
    }

    #[test]
    fn icon_selection_uses_largest_bounded_candidate() {
        let bytes = ewmh_icon(&[1, 1, 0xffffffff, 2, 1, 0xff000000, 0xffffffff]).unwrap();
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 2);
    }
}
