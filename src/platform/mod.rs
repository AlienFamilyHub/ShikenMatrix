//! 平台抽象层
//! 提供跨平台的窗口和媒体信息获取接口

use serde::{Serialize, Deserialize};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;



#[cfg(target_os = "linux")]
pub mod linux;

// 重新导出当前平台的实现
#[cfg(target_os = "macos")]
#[allow(unused_imports)]
pub use macos::*;

#[cfg(target_os = "windows")]
#[allow(unused_imports)]
pub use windows::*;

#[cfg(target_os = "linux")]
#[allow(unused_imports)]
pub use linux::*;

/// 窗口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// 窗口标题
    pub title: String,
    /// 窗口图标数据 (PNG 格式)
    pub icon_data: Option<Vec<u8>>,
    /// 进程名称
    pub process_name: String,
    /// 进程 ID
    pub pid: i32,
    /// 应用 Bundle ID (macOS) 或可执行路径
    pub app_id: Option<String>,
}

/// 平台功能 trait
///
/// 注意: PlaybackState 和 MediaMetadata 类型由各平台自行定义
#[allow(unused)]
pub trait PlatformProvider {
    /// 请求必要的权限
    fn request_permissions() -> Result<bool, String>;

    /// 检查权限状态
    fn check_permissions() -> bool;

    /// 获取当前前台窗口信息
    fn get_frontmost_window() -> Result<WindowInfo, String>;

    /// 获取所有窗口列表
    fn get_all_windows() -> Result<Vec<WindowInfo>, String>;
}
