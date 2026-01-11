//! macOS 窗口信息获取

use super::super::WindowInfo;
use super::check_accessibility_permission;
use objc2_app_kit::{NSRunningApplication, NSWorkspace, NSBitmapImageRep, NSBitmapImageFileType};
use objc2_foundation::{NSSize, NSDictionary};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 图标缓存：Bundle ID -> PNG 数据
static ICON_CACHE: Mutex<Option<HashMap<String, Vec<u8>>>> = Mutex::new(None);

/// 窗口信息缓存
struct WindowCache {
    info: Option<WindowInfo>,
    last_update: Instant,
    last_pid: i32,
}

impl Default for WindowCache {
    fn default() -> Self {
        Self {
            info: None,
            last_update: Instant::now() - Duration::from_secs(10),
            last_pid: 0,
        }
    }
}

/// 窗口信息缓存（100ms 有效期）
static WINDOW_CACHE: Mutex<Option<WindowCache>> = Mutex::new(None);
const WINDOW_CACHE_DURATION_MS: u64 = 100;

// Accessibility API
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut std::ffi::c_void;
    fn AXUIElementCopyAttributeValue(
        element: *mut std::ffi::c_void,
        attribute: *const std::ffi::c_void,
        value: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn CFRelease(cf: *mut std::ffi::c_void);
}

/// 获取当前前台窗口信息（带缓存）
pub fn get_frontmost_window_info_sync() -> Result<WindowInfo, String> {
    if !check_accessibility_permission() {
        return Err("需要辅助功能权限才能获取窗口信息".to_string());
    }

    let workspace = NSWorkspace::sharedWorkspace();
    let frontmost_app = workspace
        .frontmostApplication()
        .ok_or("无法获取前台应用")?;

    let pid = frontmost_app.processIdentifier();
    
    // 检查缓存：如果是同一个进程且缓存未过期，直接返回
    {
        let cache_guard = WINDOW_CACHE.lock().ok();
        if let Some(ref guard) = cache_guard {
            if let Some(ref cache) = **guard {
                if cache.last_pid == pid 
                    && cache.last_update.elapsed() < Duration::from_millis(WINDOW_CACHE_DURATION_MS) 
                {
                    if let Some(ref info) = cache.info {
                        return Ok(info.clone());
                    }
                }
            }
        }
    }

    // 获取新数据
    let process_name = frontmost_app
        .localizedName()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let bundle_id = frontmost_app.bundleIdentifier().map(|b| b.to_string());
    
    // 使用缓存获取图标
    let icon_data = get_cached_app_icon(&frontmost_app, bundle_id.as_deref());
    
    // 使用 Accessibility API 获取窗口标题
    let title = get_window_title_ax(pid).unwrap_or_default();

    let info = WindowInfo {
        title,
        icon_data,
        process_name,
        pid,
        app_id: bundle_id,
    };

    // 更新缓存
    if let Ok(mut cache_guard) = WINDOW_CACHE.lock() {
        let cache = cache_guard.get_or_insert_with(WindowCache::default);
        cache.info = Some(info.clone());
        cache.last_update = Instant::now();
        cache.last_pid = pid;
    }

    Ok(info)
}

/// 带缓存的图标获取
fn get_cached_app_icon(app: &NSRunningApplication, bundle_id: Option<&str>) -> Option<Vec<u8>> {
    let cache_key = bundle_id.unwrap_or("unknown").to_string();
    
    // 检查缓存
    {
        let cache = ICON_CACHE.lock().ok()?;
        if let Some(ref map) = *cache {
            if let Some(data) = map.get(&cache_key) {
                return Some(data.clone());
            }
        }
    }
    
    // 获取并缓存图标
    let icon_data = get_app_icon_png(app)?;
    
    // 存入缓存
    {
        let mut cache = ICON_CACHE.lock().ok()?;
        let map = cache.get_or_insert_with(HashMap::new);
        map.insert(cache_key, icon_data.clone());
    }
    
    Some(icon_data)
}

/// 获取应用图标数据（PNG 格式，32x32）
fn get_app_icon_png(app: &NSRunningApplication) -> Option<Vec<u8>> {
    let icon = app.icon()?;
    
    // 设置目标尺寸 32x32
    let target_size = NSSize::new(32.0, 32.0);
    
    unsafe {
        // 锁定焦点并绘制到指定尺寸
        icon.setSize(target_size);
        
        // 获取 TIFF 数据
        let tiff_data = icon.TIFFRepresentation()?;
        if tiff_data.is_empty() {
            return None;
        }
        
        // 从 TIFF 创建 NSBitmapImageRep
        let bitmap_rep = NSBitmapImageRep::imageRepWithData(&tiff_data)?;
        
        // 转换为 PNG
        let png_data = bitmap_rep.representationUsingType_properties(
            NSBitmapImageFileType::PNG,
            &NSDictionary::new(),
        )?;
        
        if png_data.is_empty() {
            return None;
        }
        
        Some(png_data.to_vec())
    }
}

/// 使用 Accessibility API 获取窗口标题
fn get_window_title_ax(pid: i32) -> Option<String> {
    unsafe {
        let app_element = AXUIElementCreateApplication(pid);
        if app_element.is_null() {
            return None;
        }

        // 获取 focused window
        let focused_window_attr = CFString::new("AXFocusedWindow");
        let mut window_ref: *mut std::ffi::c_void = std::ptr::null_mut();
        
        let result = AXUIElementCopyAttributeValue(
            app_element,
            focused_window_attr.as_concrete_TypeRef() as *const _,
            &mut window_ref,
        );

        if result != 0 || window_ref.is_null() {
            CFRelease(app_element);
            return None;
        }

        // 获取窗口标题
        let title_attr = CFString::new("AXTitle");
        let mut title_ref: *mut std::ffi::c_void = std::ptr::null_mut();
        
        let result = AXUIElementCopyAttributeValue(
            window_ref,
            title_attr.as_concrete_TypeRef() as *const _,
            &mut title_ref,
        );

        let title = if result == 0 && !title_ref.is_null() {
            let cf_string = CFString::wrap_under_create_rule(title_ref as _);
            Some(cf_string.to_string())
        } else {
            None
        };

        CFRelease(window_ref);
        CFRelease(app_element);
        
        title
    }
}
