pub fn replacer(process_name: &str) -> String {
    let cfg = crate::modules::get_config::load_config();
    for rule in cfg.rules {
        if process_name == rule.match_application {
            return rule.replace.application.clone();
        }
    }
    process_name.to_string()
}

pub fn get_window_info() -> (String, String, Vec<u8>) {
    #[cfg(target_os = "windows")]
    if cfg!(target_os = "windows") {
        return windows::get_os_windows_info();
    }

    #[cfg(target_os = "macos")]
    if cfg!(target_os = "macos") {
        match macos::get_os_macos_info() {
            Ok((process_name, _bundle_id, _path, window_title, icon_base64)) => {
                return (process_name, window_title, icon_base64);
            }
            Err(_) => {
                return (String::new(), String::new(), String::new());
            }
        }
    }

    #[cfg(target_os = "linux")] // todo：linux 平台暂时不支持
    if cfg!(target_os = "linux") {
        return (String::new(), String::new(), String::new());
    }

    (String::new(), String::new(), Vec::new())
}

#[cfg(target_os = "windows")]
pub mod windows {

    extern crate image;
    extern crate winapi;

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
    };
    use winapi::um::winuser::{LoadIconW, IDI_APPLICATION};

    use image::{DynamicImage, ImageBuffer, Rgba};
    use std::error::Error;
    use winapi::um::wingdi::{
        GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
    };
    use winapi::um::winuser::{GetDC, GetIconInfo, ICONINFO};

    pub fn get_os_windows_info() -> (String, String, Vec<u8>) {
        unsafe {
            let h_wnd = GetForegroundWindow();
            let mut window_title: [u16; 255] = [0; 255];
            GetWindowTextW(h_wnd, window_title.as_mut_ptr(), 255);

            let mut process_id: DWORD = 0;
            GetWindowThreadProcessId(h_wnd, &mut process_id);

            let process_handle: HANDLE =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);
            let mut process_name: [u16; 255] = [0; 255];
            GetModuleBaseNameW(
                process_handle,
                ptr::null_mut(),
                process_name.as_mut_ptr(),
                255,
            );
            CloseHandle(process_handle);

            let return_process_name = OsString::from_wide(&process_name)
                .to_string_lossy()
                .into_owned();
            let return_window_title = OsString::from_wide(&window_title)
                .to_string_lossy()
                .into_owned();
            let icon_png_bytes = get_window_icon(&return_window_title);
            return (return_process_name, return_window_title, icon_png_bytes);
        }
    }

    pub fn get_window_icon(window_title: &str) -> Vec<u8> {
        let hicon = get_os_windows_icon(window_title);
        match convert_hicon_to_png_bytes(hicon) {
            Ok(icon_png_bytes) => icon_png_bytes,
            Err(e) => {
                eprintln!("Failed to convert icon to PNG: {}", e);
                Vec::new()
            }
        }
    }

    pub fn get_os_windows_icon(window_title: &str) -> Option<HICON> {
        unsafe {
            let h_wnd = GetForegroundWindow();
            let mut current_window_title: [u16; 255] = [0; 255];
            GetWindowTextW(h_wnd, current_window_title.as_mut_ptr(), 255);
            let current_window_title = OsString::from_wide(&current_window_title)
                .to_string_lossy()
                .into_owned();

            if current_window_title == window_title {
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

    pub fn convert_hicon_to_png_bytes(
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
}

//
// Powered by DeepSeek.ai
//
#[cfg(target_os = "macos")]
pub mod macos {
    use objc::rc::autoreleasepool;
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    use core_foundation::{
        array::{CFArrayGetCount, CFArrayGetValueAtIndex},
        base::{CFRelease, TCFType},
        dictionary::{CFDictionaryGetValue, CFDictionaryRef},
        number::{CFNumberGetValue, CFNumberRef},
        string::{CFString, CFStringRef},
    };
    use core_graphics::window::{
        kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
        CGWindowListCopyWindowInfo,
    };
    use std::ffi::c_void;

    #[link(name = "AppKit", kind = "framework")]
    #[link(name = "Foundation", kind = "framework")]
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {}
    pub fn get_os_macos_info() -> Result<(String, String, String, String, Vec<u8>), String> {
        autoreleasepool(|| {
            unsafe {
                // 获取前台应用
                let ns_workspace_class =
                    Class::get("NSWorkspace").ok_or("NSWorkspace class not found")?;
                let shared_workspace: *mut Object = msg_send![ns_workspace_class, sharedWorkspace];
                let front_app: *mut Object = msg_send![shared_workspace, frontmostApplication];
                if front_app.is_null() {
                    return Err("Failed to get frontmost application".to_string());
                }

                // 获取应用名称
                let app_name: *mut Object = msg_send![front_app, localizedName];
                let app_name_str: *const i8 = msg_send![app_name, UTF8String];
                let process_name = std::ffi::CStr::from_ptr(app_name_str)
                    .to_string_lossy()
                    .into_owned();

                // 获取 Bundle ID
                let bundle_id: *mut Object = msg_send![front_app, bundleIdentifier];
                if bundle_id.is_null() {
                    return Err("Failed to get bundle identifier".to_string());
                }
                let bundle_id_str: *const i8 = msg_send![bundle_id, UTF8String];
                let bundle_id = std::ffi::CStr::from_ptr(bundle_id_str)
                    .to_string_lossy()
                    .into_owned();

                // 获取应用路径
                let app_url: *mut Object = msg_send![front_app, bundleURL];
                if app_url.is_null() {
                    return Err("Failed to get app URL".to_string());
                }
                let path: *mut Object = msg_send![app_url, path];
                let path_str: *const i8 = msg_send![path, UTF8String];
                let path = std::ffi::CStr::from_ptr(path_str)
                    .to_string_lossy()
                    .into_owned();

                // 获取窗口标题
                let window_list = CGWindowListCopyWindowInfo(
                    kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
                    kCGNullWindowID,
                );
                let mut window_title = String::from("Unknown");

                if !window_list.is_null() {
                    let count = CFArrayGetCount(window_list);
                    let k_window_layer = CFString::from_static_string("kCGWindowLayer");
                    let k_window_name = CFString::from_static_string("kCGWindowName");
                    let k_cfnumber_sint32_type: u32 = 3;

                    for i in 0..count {
                        let window_info = CFArrayGetValueAtIndex(window_list, i) as CFDictionaryRef;
                        let window_layer = CFDictionaryGetValue(
                            window_info,
                            k_window_layer.as_concrete_TypeRef() as *const c_void,
                        );

                        if !window_layer.is_null() {
                            let mut layer_value: i32 = 0;
                            if CFNumberGetValue(
                                window_layer as CFNumberRef,
                                k_cfnumber_sint32_type,
                                &mut layer_value as *mut _ as *mut c_void,
                            ) && layer_value == 0
                            {
                                let window_name_ptr = CFDictionaryGetValue(
                                    window_info,
                                    k_window_name.as_concrete_TypeRef() as *const c_void,
                                )
                                    as CFStringRef;

                                if !window_name_ptr.is_null() {
                                    window_title =
                                        CFString::wrap_under_get_rule(window_name_ptr).to_string();
                                    break;
                                }
                            }
                        }
                    }
                    CFRelease(window_list as *mut c_void);
                }

                // 获取应用图标
                let icon: *mut Object = msg_send![front_app, icon];
                if icon.is_null() {
                    return Err("Failed to get app icon".to_string());
                }

                let tiff_data: *mut Object = msg_send![icon, TIFFRepresentation];
                let ns_bitmap_image_rep_class =
                    Class::get("NSBitmapImageRep").ok_or("NSBitmapImageRep class not found")?;
                let bitmap_rep: *mut Object =
                    msg_send![ns_bitmap_image_rep_class, imageRepWithData: tiff_data];

                let ns_png_file_type = 1;
                let properties: *mut Object = msg_send![Class::get("NSDictionary").unwrap(), new];
                let png_data: *mut Object = msg_send![bitmap_rep, representationUsingType: ns_png_file_type properties: properties];

                let length: usize = msg_send![png_data, length];
                let mut buffer = vec![0u8; length];
                let _: () = msg_send![png_data, getBytes: buffer.as_mut_ptr() length: length];

                Ok((process_name, bundle_id, path, window_title, buffer))
            }
        })
    }
}
