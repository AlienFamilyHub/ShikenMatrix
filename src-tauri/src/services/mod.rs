//! 业务服务层
//! 包含数据上报、状态管理等业务逻辑

pub mod config;
pub mod reporter;

pub use config::{load_config, save_close_behavior, save_reporter_config, CloseBehavior};
pub use reporter::{LogLevel, Monitor, Reporter, ReporterConfig, ReporterEvent};
