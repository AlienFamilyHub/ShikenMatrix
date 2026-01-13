#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod libs;
mod modules;

use libs::{
    cache::create_cache_directory,
    commands::{get_config, get_version, open_log_directory, save_config, start},
    create_config::create_config_file,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, Runtime, WindowEvent,
};

const VERSION: &str = "0.0.3";

/// 初始化应用程序日志和配置文件
fn initialize_app() -> Result<(), Box<dyn std::error::Error>> {
    crate::modules::logs::init_logger();
    log::info!("KizunaBaka {}", VERSION);
    
    create_cache_directory()?;
    create_config_file()?;
    
    Ok(())
}

/// 创建系统托盘菜单
fn create_tray_menu<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<Menu<R>> {
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?,
            &MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
        ],
    )
}

/// 设置主窗口关闭行为
fn setup_window_behavior<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            window_clone.hide().unwrap_or_else(|e| {
                log::error!("Failed to hide window: {}", e);
            });
        }
    });
}

/// 设置系统托盘
fn setup_tray<R: Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let tray_menu = create_tray_menu(app)?;
    
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&tray_menu)
        .tooltip("KizunaBaka")
        .on_menu_event(|app, event| {
            if let Some(main_window) = app.get_webview_window("main") {
                match event.id.0.as_str() {
                    "show" => {
                        if let Err(e) = main_window.show().and_then(|_| main_window.set_focus()) {
                            log::error!("Failed to show window: {}", e);
                        }
                    }
                    "hide" => {
                        if let Err(e) = main_window.hide() {
                            log::error!("Failed to hide window: {}", e);
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                }
            } else {
                log::error!("Main window not found");
            }
        })
        .build(app)?;
    
    Ok(())
}

pub fn main() {
    if let Err(e) = initialize_app() {
        eprintln!("Application initialization failed: {}", e);
        std::process::exit(1);
    }

    log::info!("开始尝试初始化");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 设置主窗口行为
            log::info!("开始尝试设置主窗口行为");
            if let Some(main_window) = app.get_webview_window("main") {
                setup_window_behavior(&main_window);
            } else {
                log::error!("Main window not found during setup");
            }
            
            // 设置系统托盘
            log::info!("开始尝试设置系统托盘");
            setup_tray(app)?;
            
            log::info!("初始化完成，开始尝试运行上报函数");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start,
            open_log_directory,
            get_version,
            save_config,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}