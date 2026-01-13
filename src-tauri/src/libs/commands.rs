use crate::libs::report;
use crate::modules::get_config::MainConfig;
use tauri::Emitter;
use tauri::{AppHandle, Wry};

#[tauri::command]
pub fn start(app_handle: AppHandle<Wry>) {

    log::info!("Start KizunaBaka Services");

    let config = crate::modules::get_config::load_config();
    let endpoint = config.server_config.endpoint.clone();
    let token = config.server_config.token.clone();
    let report_time = std::time::Duration::from_secs(config.server_config.report_time as u64);

    std::thread::spawn(move || loop {
        std::thread::sleep(report_time);
        let (logdata, data, icon_data, _media_update, thumbnail) =
            report::report(&endpoint, &token);
        let home_event_data = serde_json::json!({
            "data": data,
            "icon": icon_data,
            "AlbumThumbnail": thumbnail,
        });
        app_handle
            .emit("home-event", home_event_data)
            .unwrap_or_else(|e| {
                log::error!("Failed to emit home-event: {}", e);
            });
        app_handle.emit("log-event", logdata).unwrap_or_else(|e| {
            log::error!("Failed to emit log-event: {}", e);
        });
    });
}

#[tauri::command]
pub fn open_log_directory() {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("explorer")
            .arg("./logs/")
            .spawn()
        {
            eprintln!("Failed to open log directory: {}", e);
        }
    }
}

#[tauri::command]
pub fn save_config(config: String, app_handle: AppHandle<Wry>) {
    let config: MainConfig = serde_json::from_str(&config).unwrap();
    let config_path = std::env::current_dir().unwrap().join(if cfg!(dev) {
        "../config.yml"
    } else {
        "/config.yml"
    });
    let config_data = serde_yaml::to_string(&config).unwrap();
    std::fs::write(config_path, config_data).unwrap();

    app_handle.emit("config-updated", ()).unwrap_or_else(|e| {
        eprintln!("Failed to emit config-updated event: {}", e);
    });
}

#[tauri::command]
pub fn get_config() -> String {
    let config = crate::modules::get_config::load_config();
    serde_json::to_string(&config).unwrap()
}

#[tauri::command]
pub fn get_version() -> String {
    crate::VERSION.to_string()
}
