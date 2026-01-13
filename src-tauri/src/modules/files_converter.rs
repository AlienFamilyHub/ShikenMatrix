pub fn _convert_png_to_base64(png_data: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    STANDARD.encode(png_data)
}

pub fn convert_to_webp(image_data: &[u8]) -> Result<Vec<u8>, String> {
    use image::io::Reader as ImageReader;
    use image::ImageFormat;
    use std::io::Cursor;

    let img = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .map_err(|e| format!("无法识别图像格式: {}", e))?
        .decode()
        .map_err(|e| format!("解码图像失败: {}", e))?;

    let mut webp_data = Vec::new();
    let mut cursor = Cursor::new(&mut webp_data);

    img.write_to(&mut cursor, ImageFormat::WebP)
        .map_err(|e| format!("转换为WebP失败: {}", e))?;

    Ok(webp_data)
}

pub fn encode_as_base64(data: &[u8]) -> String {
    use base64::Engine as Base64Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(data);
    // 更全面的图片类型检测
    let mime_type = if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        "image/gif"
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp"
    } else {
        // 默认情况下假定为jpeg
        "image/jpeg"
    };
    format!("data:{};base64,{}", mime_type, base64_data)
}
