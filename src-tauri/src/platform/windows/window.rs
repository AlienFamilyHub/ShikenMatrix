//! Windows 窗口信息获取模块

use crate::platform::WindowInfo;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::windef::HICON;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleBaseNameW;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winnt::PROCESS_VM_READ;
use winapi::um::winuser::{
    GetClassLongPtrW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    SendMessageW, GCLP_HICON, GCLP_HICONSM, ICON_BIG, ICON_SMALL, WM_GETICON,
    LoadIconW, IDI_APPLICATION, GetDC, GetIconInfo, ICONINFO,
};
use winapi::um::wingdi::{
    GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::error::Error;

/// 获取当前前台窗口信息
pub fn get_frontmost_window() -> Result<WindowInfo, String> {
    unsafe {
        let h_wnd = GetForegroundWindow();
        if h_wnd.is_null() {
            return Err("No foreground window".to_string());
        }

        let mut window_title: [u16; 255] = [0; 255];
        GetWindowTextW(h_wnd, window_title.as_mut_ptr(), 255);

        let mut process_id: DWORD = 0;
        GetWindowThreadProcessId(h_wnd, &mut process_id);

        let process_handle: HANDLE =
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);
        
        let mut process_name: [u16; 255] = [0; 255];
        if !process_handle.is_null() {
            GetModuleBaseNameW(
                process_handle,
                ptr::null_mut(),
                process_name.as_mut_ptr(),
                255,
            );
            CloseHandle(process_handle);
        }

        let process_name_str = OsString::from_wide(&process_name)
            .to_string_lossy()
            .into_owned()
            .trim_matches(char::from(0))
            .to_string();
            
        let window_title_str = OsString::from_wide(&window_title)
            .to_string_lossy()
            .into_owned()
            .trim_matches(char::from(0))
            .to_string();

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

/// 获取所有窗口列表 (目前仅返回前台窗口作为占位，或者留空)
pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    // 暂时只实现获取前台窗口，因为参考代码只提供了获取前台窗口的逻辑
    match get_frontmost_window() {
        Ok(info) => Ok(vec![info]),
        Err(_) => Ok(Vec::new()),
    }
}

fn get_window_icon(window_title: &str) -> Vec<u8> {
    let hicon = get_os_windows_icon(window_title);
    match convert_hicon_to_png_bytes(hicon) {
        Ok(icon_png_bytes) => icon_png_bytes,
        Err(_e) => {
            // eprintln!("Failed to convert icon to PNG: {}", e);
            Vec::new()
        }
    }
}

fn get_os_windows_icon(window_title: &str) -> Option<HICON> {
    unsafe {
        let h_wnd = GetForegroundWindow();
        let mut current_window_title: [u16; 255] = [0; 255];
        GetWindowTextW(h_wnd, current_window_title.as_mut_ptr(), 255);
        let current_window_title_str = OsString::from_wide(&current_window_title)
            .to_string_lossy()
            .into_owned()
            .trim_matches(char::from(0))
            .to_string();

        // 简单的校验，确保是同一个窗口
        if current_window_title_str == window_title {
            let h_icon = SendMessageW(h_wnd, WM_GETICON, ICON_BIG as usize, 0) as HICON;
            if h_icon.is_null() || h_icon == LoadIconW(ptr::null_mut(), IDI_APPLICATION) {
                let h_icon = SendMessageW(h_wnd, WM_GETICON, ICON_SMALL as usize, 0) as HICON;
                if h_icon.is_null() || h_icon == LoadIconW(ptr::null_mut(), IDI_APPLICATION) {
                    let h_icon = GetClassLongPtrW(h_wnd, GCLP_HICON) as HICON;
                    if h_icon.is_null() || h_icon == LoadIconW(ptr::null_mut(), IDI_APPLICATION)
                    {
                        let h_icon = GetClassLongPtrW(h_wnd, GCLP_HICONSM) as HICON;
                        if h_icon.is_null()
                            || h_icon == LoadIconW(ptr::null_mut(), IDI_APPLICATION)
                        {
                            let h_icon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);
                            return Some(h_icon);
                        }
                        return Some(h_icon);
                    }
                    return Some(h_icon);
                }
                return Some(h_icon);
            }
            return Some(h_icon);
        }
    }
    None
}

fn convert_hicon_to_png_bytes(
    hicon_option: Option<HICON>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    if let Some(hicon) = hicon_option {
        unsafe {
            // Get icon info
            let mut icon_info: ICONINFO = std::mem::zeroed();
            if GetIconInfo(hicon, &mut icon_info) == 0 {
                return Err("Failed to get icon info".into());
            }
            let hbitmap = icon_info.hbmColor;
            if hbitmap.is_null() {
                return Err("Icon does not have a color bitmap".into());
            }

            // Get bitmap information
            let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
            let mut bitmap = BITMAP {
                bmType: 0,
                bmWidth: 0,
                bmHeight: 0,
                bmWidthBytes: 0,
                bmPlanes: 0,
                bmBitsPixel: 0,
                bmBits: ptr::null_mut(),
            };
            if GetObjectW(
                hbitmap as _,
                std::mem::size_of::<BITMAP>() as _,
                &mut bitmap as *mut _ as _,
            ) == 0
            {
                return Err("Failed to get bitmap object".into());
            }

            // Setup BITMAPINFOHEADER
            bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bitmap_info.bmiHeader.biWidth = bitmap.bmWidth;
            bitmap_info.bmiHeader.biHeight = -(bitmap.bmHeight); // Negative to get top-down image
            bitmap_info.bmiHeader.biPlanes = 1;
            bitmap_info.bmiHeader.biBitCount = 32;
            bitmap_info.bmiHeader.biCompression = 0; // BI_RGB
            bitmap_info.bmiHeader.biSizeImage = 0;
            bitmap_info.bmiHeader.biClrUsed = 0;
            bitmap_info.bmiHeader.biClrImportant = 0;

            // Allocate buffer and get bitmap bits
            let mut buffer = vec![0u8; (bitmap.bmWidth * bitmap.bmHeight * 4) as usize];
            if GetDIBits(
                GetDC(ptr::null_mut()),
                hbitmap,
                0,
                bitmap.bmHeight as u32,
                buffer.as_mut_ptr() as *mut _,
                &mut bitmap_info,
                DIB_RGB_COLORS,
            ) == 0
            {
                return Err("Failed to get bitmap bits".into());
            }

            // Convert BGRA to RGBA
            for i in 0..bitmap.bmWidth * bitmap.bmHeight {
                let offset = (i * 4) as usize;
                buffer.swap(offset, offset + 2); // Swap blue (B) and red (R)
            }

            // Create image buffer and encode as PNG in memory
            let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
                bitmap.bmWidth as u32,
                bitmap.bmHeight as u32,
                buffer,
            )
            .ok_or("Failed to create image buffer")?;
            let dynamic_image = DynamicImage::ImageRgba8(img);

            let mut bytes = Vec::new();
            dynamic_image.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )?;
            Ok(bytes)
        }
    } else {
        Err("No HICON provided".into())
    }
}
