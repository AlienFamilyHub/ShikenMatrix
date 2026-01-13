use chrono::Local;
use log::{info, LevelFilter};
use std::fs::create_dir_all;
use std::io::Write;
use std::path::Path;

// 初始化日志系统
pub fn init_logger() {
    let log_dir = if cfg!(dev) { "../logs" } else { "/logs" };

    if !Path::new(log_dir).exists() {
        create_dir_all(log_dir).expect("无法创建日志目录");
    }

    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let log_file_path = format!("{}/{}.log", log_dir, date_str);

    // 配置环境变量，如果没有设置则默认为 debug 级别
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    // 初始化日志记录器
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("无法打开日志文件");

    // 创建一个同时写入文件和标准输出的 logger
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .target(env_logger::Target::Stdout) // 添加标准输出目标
        .write_style(env_logger::WriteStyle::Always)
        .init();

    info!("日志系统初始化完成");
}

pub fn _get_today_log() -> String {
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let log_dir = if cfg!(dev) { "../logs" } else { "/logs" };
    let log_file_path = format!("{}/{}.log", log_dir, date_str);

    if Path::new(&log_file_path).exists() {
        std::fs::read_to_string(log_file_path).expect("无法读取日志文件")
    } else {
        String::from("今日无日志记录")
    }
}
