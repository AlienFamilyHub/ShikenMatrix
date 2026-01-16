//! Windows 窗口信息获取模块

use crate::platform::WindowInfo;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HWND, CloseHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;

/// 获取当前前台窗口信息
pub fn get_frontmost_window() -> Result<WindowInfo, String> {
    unsafe {
        let h_wnd = HWND(GetForegroundWindow().0);
        if h_wnd.is_invalid() {
            return Err("No foreground window".to_string());
        }

        let mut window_title: [u16; 255] = [0; 255];
        GetWindowTextW(h_wnd, &mut window_title);

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(h_wnd, Some(&mut process_id));

        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id
        );

        let mut process_name: [u16; 255] = [0; 255];
        let process_name_str = if let Ok(handle) = process_handle {
            let result = GetModuleBaseNameW(
                handle,
                None,
                &mut process_name
            );
            let _ = CloseHandle(handle);

            if result > 0 && process_name[0] != 0 {
                OsString::from_wide(&process_name)
                    .to_string_lossy()
                    .into_owned()
                    .trim_matches(char::from(0))
                    .to_string()
            } else {
                String::from("Unknown")
            }
        } else {
            String::from("Unknown")
        };

        let window_title_str = OsString::from_wide(&window_title)
            .to_string_lossy()
            .into_owned()
            .trim_matches(char::from(0))
            .to_string();

        // 获取窗口图标
        let icon_data = get_window_icon(&window_title_str);

        Ok(WindowInfo {
            title: window_title_str,
            icon_data: if icon_data.is_empty() { None } else { Some(icon_data) },
            process_name: process_name_str.clone(),
            pid: process_id as i32,
            app_id: Some(process_name_str), // 使用进程名作为 app_id
        })
    }
}

/// 获取所有窗口列表 (目前仅返回前台窗口)
pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    // 暂时只实现获取前台窗口
    match get_frontmost_window() {
        Ok(info) => Ok(vec![info]),
        Err(_) => Ok(Vec::new()),
    }
}

fn get_window_icon(_window_title: &str) -> Vec<u8> {
    // 简化实现：暂时返回空，避免复杂的图标处理
    // TODO: 实现完整的图标获取逻辑
    Vec::new()
}
