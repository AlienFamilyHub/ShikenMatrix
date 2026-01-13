use std::fs;
use std::path::{Path, PathBuf};

pub fn create_cache_directory() -> std::io::Result<()> {
    // 根据开发模式选择路径
    let cache_path = if cfg!(dev) {
        Path::new("..").join("cache")
    } else {
        Path::new("cache").to_path_buf()
    };

    // 检查目录是否存在，若不存在则创建
    if !fs::metadata(&cache_path).is_ok() {
        fs::create_dir(&cache_path)?;
    }

    Ok(())
}

pub fn get_cache_directory() -> PathBuf {
    if cfg!(dev) {
        return Path::new("..").join("cache");
    } else {
        return Path::new("cache").to_path_buf();
    }
}
