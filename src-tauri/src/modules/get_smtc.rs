#[cfg(target_os = "windows")]
pub mod windows {
    use std::fs::File;
    use std::io::Write;
    use tokio::runtime::Runtime;
    use windows::core::Result;
    use windows::core::HSTRING;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackInfo,
        GlobalSystemMediaTransportControlsSessionTimelineProperties,
    };
    use windows::Storage::Streams::DataReader;

    pub fn get_media_info() -> (String, String, String, Vec<u8>, i64, i64) {
        // 获取配置，检查是否需要上报SMTC信息
        let config = crate::modules::get_config::load_config();
        if !config.server_config.report_smtc {
            return (
                "".to_string(),
                "".to_string(),
                "".to_string(),
                Vec::new(),
                0,
                0,
            );
        }

        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        match rt.block_on(async_main()) {
            Ok(info) => info,
            Err(e) => {
                if e.code().is_err() {
                    log::error!("应该是成功执行了操作: {}", e);
                    (
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        Vec::new(),
                        0,
                        0,
                    )
                } else {
                    log::error!("Failed to get SMTC info: {}", e);
                    (
                        "".to_string(),
                        "".to_string(),
                        "".to_string(),
                        Vec::new(),
                        0,
                        0,
                    )
                }
            }
        }
    }

    async fn async_main() -> Result<(String, String, String, Vec<u8>, i64, i64)> {
        let session_manager =
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;
        let current_session = session_manager.GetCurrentSession()?;

        // 获取播放信息
        let playback_info: GlobalSystemMediaTransportControlsSessionPlaybackInfo =
            current_session.GetPlaybackInfo()?;
        let playback_status = playback_info.PlaybackStatus()?;
        // 检查播放状态，如果是暂停或其他非播放状态，返回空值
        if playback_status
            != windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
        {
            return Ok((
                "".to_string(),
                "".to_string(),
                "".to_string(),
                Vec::new(),
                0,
                0,
            ));
        }

        // 获取时间线信息（包含歌曲长度和当前播放位置）
        let timeline_properties: GlobalSystemMediaTransportControlsSessionTimelineProperties =
            current_session.GetTimelineProperties()?;
        let duration_ticks = timeline_properties.EndTime()?.Duration;
        let elapsed_time_ticks = timeline_properties.Position()?.Duration;

        const TICKS_PER_SECOND: i64 = 10_000_000;

        let duration = (duration_ticks as f64 / TICKS_PER_SECOND as f64).round() as i64;
        let elapsed_time = (elapsed_time_ticks as f64 / TICKS_PER_SECOND as f64).round() as i64;

        // 获取媒体属性
        let media_properties_operation = current_session.TryGetMediaPropertiesAsync()?;
        let media_properties = media_properties_operation.get()?;

        let source_app_name_hstring: HSTRING = current_session.SourceAppUserModelId()?.into();
        let title_hstring: HSTRING = media_properties.Title()?.into();
        let artist_hstring: HSTRING = media_properties.Artist()?.into();

        let thumbnail_ref = media_properties.Thumbnail()?;
        let thumbnail_stream = thumbnail_ref.OpenReadAsync()?.get()?;
        // 使用 DataReader 读取流中的数据
        let data_reader = DataReader::CreateDataReader(&thumbnail_stream)?;
        let stream_size = thumbnail_stream.Size()?;
        // 将 u64 转换为 u32，处理可能的溢出
        let stream_size_u32: u32 = stream_size.try_into().unwrap_or(0);

        let mut thumbnail_data = Vec::new();

        if stream_size_u32 > 0 {
            data_reader.LoadAsync(stream_size_u32)?.get()?;
            // 读取字节数据
            thumbnail_data.resize(stream_size_u32 as usize, 0);
            data_reader.ReadBytes(&mut thumbnail_data)?;

            // 保存本地缓存
            let cache_path = crate::libs::cache::get_cache_directory();
            let cache_file_path = cache_path.join("album_thumbnail.png");

            // 写入文件
            let mut file = File::create(&cache_file_path)?;
            file.write_all(&thumbnail_data)?;
        }

        Ok((
            title_hstring.to_string_lossy().to_owned(),
            artist_hstring.to_string_lossy().to_owned(),
            source_app_name_hstring.to_string_lossy().to_owned(),
            thumbnail_data,
            duration,
            elapsed_time,
        ))
    }
}
