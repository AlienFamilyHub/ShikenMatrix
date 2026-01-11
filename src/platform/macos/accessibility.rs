//! macOS Accessibility API 权限管理

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

// Accessibility API 外部函数声明
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const core_foundation::dictionary::__CFDictionary) -> bool;
}

// kAXTrustedCheckOptionPrompt 键
const AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";

/// 检查是否已获得辅助功能权限
pub fn check_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// 请求辅助功能权限
/// 
/// 如果未授权，会弹出系统权限请求对话框
/// 返回当前是否已授权
pub fn request_accessibility_permission() -> Result<bool, String> {
    unsafe {
        // 创建选项字典，设置 prompt = true 以显示系统对话框
        let key = CFString::new(AX_TRUSTED_CHECK_OPTION_PROMPT);
        let value = CFBoolean::true_value();

        let pairs = [(key, value)];
        let options = CFDictionary::from_CFType_pairs(&pairs);

        let is_trusted = AXIsProcessTrustedWithOptions(
            options.as_concrete_TypeRef() as *const _
        );

        if is_trusted {
            Ok(true)
        } else {
            // 返回 false 表示需要用户手动授权
            // 系统会自动弹出提示框引导用户到系统设置
            Ok(false)
        }
    }
}

/// 打开系统偏好设置中的辅助功能页面
#[allow(unused)]
pub fn open_accessibility_preferences() -> Result<(), String> {
    use std::process::Command;

    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| format!("无法打开系统偏好设置: {}", e))?;

    Ok(())
}
