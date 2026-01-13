pub fn get_media_info() -> (String, String, String, Vec<u8>, i64, i64) {
    #[cfg(target_os = "windows")]
    if cfg!(target_os = "windows") {
        return crate::modules::get_smtc::windows::get_media_info();
    }

    // mac的媒体信息获取怪怪的，求help
    return (
        String::new(),
        String::new(),
        String::new(),
        Vec::new(),
        0,
        0,
    );
}
