use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};

pub mod platform;
pub mod services;

use services::{
    load_config, save_close_behavior, save_reporter_config, CloseBehavior, Monitor, Reporter,
    ReporterConfig,
};

struct AppState {
    monitor: Arc<Mutex<Option<Monitor>>>,
    reporter: Arc<Mutex<Option<Reporter>>>,
}

#[derive(serde::Serialize)]
struct PermissionStatus {
    accessibility: bool,
    media: bool,
}

#[derive(serde::Serialize)]
struct ConnectionStatus {
    is_monitoring: bool,
    is_reporting: bool,
    is_connected: bool,
    last_error: Option<String>,
}

#[tauri::command]
fn get_config() -> ReporterConfig {
    load_config().reporter
}

#[tauri::command]
fn save_config(config: ReporterConfig) -> Result<(), String> {
    save_reporter_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn apply_close_decision(
    app: AppHandle,
    behavior: CloseBehavior,
    remember: bool,
) -> Result<(), String> {
    if remember {
        save_close_behavior(behavior.clone()).map_err(|e| e.to_string())?;
    }

    match behavior {
        CloseBehavior::HideToTray => hide_main_window(&app),
        CloseBehavior::Quit => {
            app.exit(0);
            Ok(())
        }
    }
}

#[tauri::command]
fn start_monitor(
    app: AppHandle,
    config: ReporterConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut monitor_guard = state.monitor.lock().unwrap();

    // Stop existing monitor (and any attached reporter)
    if let Some(old) = monitor_guard.take() {
        old.stop();
        drop(old);
    }
    // Also clean up reporter reference
    {
        let mut reporter_guard = state.reporter.lock().unwrap();
        if let Some(old_r) = reporter_guard.take() {
            old_r.stop();
            drop(old_r);
        }
    }

    // Create mpsc channel to receive events
    let (tx, rx) = std::sync::mpsc::channel();
    let monitor = Monitor::new(config, Some(tx));
    *monitor_guard = Some(monitor);

    // Spawn thread to forward events to Tauri frontend
    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            let _ = app.emit("reporter-event", event);
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_monitor(state: State<'_, AppState>) {
    // Stop reporter first
    {
        let mut reporter_guard = state.reporter.lock().unwrap();
        if let Some(r) = reporter_guard.take() {
            r.stop();
        }
    }
    // Then stop monitor
    {
        let mut monitor_guard = state.monitor.lock().unwrap();
        if let Some(m) = monitor_guard.take() {
            m.stop();
        }
    }
}

#[tauri::command]
fn start_reporter(config: ReporterConfig, state: State<'_, AppState>) -> Result<(), String> {
    let monitor_guard = state.monitor.lock().unwrap();
    let monitor = monitor_guard
        .as_ref()
        .ok_or_else(|| "请先启动监控".to_string())?;

    let mut reporter_guard = state.reporter.lock().unwrap();

    // Stop existing reporter
    if let Some(old) = reporter_guard.take() {
        monitor.detach_reporter();
        old.stop();
        drop(old);
    }

    // Create new reporter — it shares artwork_urls with monitor
    let reporter = Reporter::new(config, monitor.artwork_urls());
    monitor.attach_reporter(&reporter);

    *reporter_guard = Some(reporter);

    Ok(())
}

#[tauri::command]
fn stop_reporter(state: State<'_, AppState>) {
    let monitor_guard = state.monitor.lock().unwrap();
    if let Some(m) = monitor_guard.as_ref() {
        m.detach_reporter();
    }
    let mut reporter_guard = state.reporter.lock().unwrap();
    if let Some(r) = reporter_guard.take() {
        r.stop();
    }
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> ConnectionStatus {
    let monitor_guard = state.monitor.lock().unwrap();
    let reporter_guard = state.reporter.lock().unwrap();

    let is_monitoring = monitor_guard.is_some();
    let (is_reporting, is_connected, last_error) = if let Some(r) = reporter_guard.as_ref() {
        (true, r.is_connected(), r.last_error())
    } else {
        (false, false, None)
    };

    ConnectionStatus {
        is_monitoring,
        is_reporting,
        is_connected,
        last_error,
    }
}

#[tauri::command]
fn check_permissions() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    let accessibility = platform::macos::check_accessibility_permission();
    #[cfg(not(target_os = "macos"))]
    let accessibility = true;

    PermissionStatus {
        accessibility,
        media: true,
    }
}

#[tauri::command]
fn request_permissions() {
    #[cfg(target_os = "macos")]
    let _ = platform::macos::request_accessibility_permission();
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}

fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        app.set_dock_visibility(true);
        set_macos_application_icon();
    }

    let show_item = MenuItemBuilder::with_id("show_main_window", "打开主窗口").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit_app", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &quit_item])
        .build()?;

    let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    let tray_builder = TrayIconBuilder::new()
        .tooltip("ShikenMatrix")
        .menu(&menu)
        .icon(tray_icon)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_main_window" => show_main_window(app),
            "quit_app" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    tray_builder.build(app)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_macos_application_icon() {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let marker = unsafe { MainThreadMarker::new_unchecked() };
    let app = NSApplication::sharedApplication(marker);
    let icon_data = NSData::with_bytes(include_bytes!("../icons/icon.png"));
    if let Some(app_icon) = NSImage::initWithData(NSImage::alloc(), &icon_data) {
        unsafe { app.setApplicationIconImage(Some(&app_icon)) };
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(setup_tray)
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match load_config().close_behavior {
                    Some(CloseBehavior::HideToTray) => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    Some(CloseBehavior::Quit) => {}
                    None => {
                        api.prevent_close();
                        let _ = window.emit("close-behavior-requested", ());
                    }
                }
            }
        })
        .manage(AppState {
            monitor: Arc::new(Mutex::new(None)),
            reporter: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            apply_close_decision,
            start_monitor,
            stop_monitor,
            start_reporter,
            stop_reporter,
            get_status,
            check_permissions,
            request_permissions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
