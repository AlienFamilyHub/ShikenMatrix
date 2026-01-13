use std::fs;
use std::io::Write;

pub const DEFAULT_CONFIG: &str = r#"
server_config:
  endpoint: "apiurl" # https://api.example.com/api/v2/fn/ps/update
  token: "apikey" # 设置的key
  report_time: 5 # 上报时间间隔，单位秒
  report_smtc: true # 是否上报SMTC信息
  skip_smtc_cover: false # 是否跳过SMTC封面上传
  upload_smtc_cover: false # 是否将SMTC封面上传到S3服务器
  log_base64: false # 是否将base64的SMTC封面信息写入日志，默认判断条件：log_base64 && report_smtc && !skip_smtc_cover && !upload_smtc_cover
  s3_config:
    s3_enable: false # 是否启用S3功能
    upload_path: "" # 自定义URL路径，支持变量：{year}年、{month}月、{day}日，仅能定制URL目录路径，暂且不支持对文件名进行定制
    endpoint: "" # S3端点
    region: "" # S3区域
    bucket_name: "" # S3桶名称
    access_key: "" # S3访问密钥
    secret_key: "" # S3密钥
    custom_url: "" # S3自定义URL
rules: # 软件名的替换规则
  - match_application: WeChat
    replace:
      application: 微信
      description: 一个小而美的办公软件
  - match_application: QQ
    replace:
      application: QQ
      description: 一个多功能的通讯软件
  - match_application: Netease Cloud Music
    replace:
      application: 网易云音乐
      description: 一个音乐播放和分享的平台
"#;

pub fn create_config_file() -> std::io::Result<()> {
    use std::path::Path;

    let config_file = if cfg!(dev) {
        Path::new("..")
            .join("config.yml")
            .to_str()
            .unwrap()
            .to_string()
    } else {
        "config.yml".to_string()
    };

    // 检查文件是否存在
    if !fs::metadata(&config_file).is_ok() {
        let mut file = fs::File::create(&config_file)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
    }

    Ok(())
}
