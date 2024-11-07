use std::path::PathBuf;

use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use serde::{Deserialize, Deserializer};

use cfg_lib::{conf};
use crate::serde_default;

/// 通过配置文件控制日志格式化输出
/// # Examples
///
///  ```yaml
/// log:
///   level: warn #全局日志等级 可选：默认 info
///   prefix: server #全局日志文件前缀; 可选：默认 app 指定生成日志文件添加日期后缀，如 server_2024-10-26.log
///   store_path: ./logs #日志文件根目录；可选 默认 当前目录
///   specify: #指定日志输出 可选，不指定日志
///     - crate_name: test_log::a  #或者test_log用指全部  必选
///       level: debug #日志等级 必选
///       file_name_prefix: a #日志文件前缀 可选 当未指定时，记录到全局日志文件中，等级由指定日志等级控制
///       additivity: false #是否记录到全局日志文件中 可选 默认false,全局日志文件会再次根据全局日志等级过滤记录
///     - crate_name: test_log::b  #或者test_log用指全部
///       level: debug #日志等级
///     - crate_name: test_log::c  #或者test_log用指全部
///       level: debug #日志等级
///       file_name_prefix: c #日志文件前缀
///       additivity: true #是否记录到全局日志文件中
///  ```
#[derive(Debug, Deserialize)]
#[conf(prefix = "log")]
pub struct Logger {
    #[serde(default)]
    store_path: PathBuf,
    #[serde(default = "default_prefix")]
    prefix: String,
    #[serde(deserialize_with = "validate_level")]
    #[serde(default = "default_level")]
    level: String,
    specify: Option<Vec<Specify>>,
}
serde_default!(default_prefix, String, "app".to_string());
serde_default!(default_level, String, "info".to_string());

#[derive(Debug, Deserialize)]
pub struct Specify {
    crate_name: String,
    #[serde(deserialize_with = "validate_level")]
    level: String,
    file_name_prefix: Option<String>,
    additivity: Option<bool>,
}


impl Logger {
    pub fn init() {
        let mut log: Logger = Logger::conf();
        if !log.store_path.ends_with("/") { log.store_path.push("") };
        let default_level: LevelFilter = level_filter(&log.level);
        let path = std::path::Path::new(&log.store_path);
        std::fs::create_dir_all(path).unwrap();
        let mut add_crate = Vec::new();
        let mut dispatch = Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] [{}] {} {}\n{}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.target(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    message,
                ))
            });
        if let Some(specify) = &log.specify {
            for s in specify {
                let module_level = level_filter(&s.level);
                let target = s.crate_name.clone();

                // 根据 `additivity` 决定是否记录到默认日志
                if !s.additivity.unwrap_or(false) {
                    add_crate.push(target.clone());
                }

                // 为特定模块创建日志输出
                let mut module_dispatch = Dispatch::new()
                    .level(module_level)
                    .filter(move |metadata| metadata.target().starts_with(&target));

                // 如果指定了文件名前缀，则将日志输出到指定的文件
                let prefix = s.file_name_prefix.as_deref().unwrap_or(&log.prefix);
                module_dispatch = module_dispatch
                    .chain(std::io::stdout())
                    .chain(fern::DateBased::new(&log.store_path, format!("{}_{}.log", prefix, "%Y-%m-%d".to_string())));

                dispatch = dispatch.chain(module_dispatch);
            }
        }

        // 配置默认日志输出
        let default_dispatch = Dispatch::new()
            .level(default_level)
            .filter(move |metadata| !add_crate.iter().any(|t| metadata.target().starts_with(t)))
            .chain(std::io::stdout())
            .chain(fern::DateBased::new(&log.store_path, format!("{}_{}.log", log.prefix, "%Y-%m-%d".to_string())));

        dispatch
            .chain(default_dispatch)
            .apply()
            .expect("Logger initialization failed");
    }
}

pub fn level_filter(level: &str) -> LevelFilter {
    match level.trim().to_uppercase().as_str() {
        "OFF" => LevelFilter::Off,
        "ERROR" => LevelFilter::Error,
        "WARN" => LevelFilter::Warn,
        "INFO" => LevelFilter::Info,
        "DEBUG" => LevelFilter::Debug,
        "TRACE" => LevelFilter::Trace,
        _ => panic!("The log level is invalid"),
    }
}

pub fn validate_level<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let level = String::deserialize(deserializer)?;
    level_filter(&*level);
    Ok(level)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_yaml::Value;
    use super::*;

    #[test]
    fn test_default_value() {
        println!("{:?}", PathBuf::default());
    }

    const YML: &str = r#"
db:
  mysql:
    host_or_ip: "localhost"
    port: 3306
    db_name: "test_db"
    user: "root"
    pass: "password"
  sqllite:
    host_or_ip: "localhost"
    port: 3306
    db_name: "test_db"
    user: "root"
    pass: "password"
"#;

    #[test]
    fn test_prefix_conf() {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(YML).unwrap();
        let mut target_value = &yaml_value;
        for key in "db.mysql".split('.') {
            if let serde_yaml::Value::Mapping(map) = target_value {
                target_value = map.get(Value::String(key.to_string()))
                    .expect("Specified prefix not found in YAML");
            } else {
                panic!("Invalid YAML structure for the specified prefix");
            }
        }
        let db: DbModel = serde_yaml::from_value(target_value.clone()).unwrap();
        println!("{:?}", db);
    }

    #[derive(Debug, Deserialize)]
    #[conf]
    pub struct DbModel
    {
        pub host_or_ip: String,
        pub port: u16,
        pub db_name: String,
        pub user:
            Option<String>,
        pub pass: Option<String>,
        pub connect_attrs:
            Option<HashMap<String, String>>,
    }

    #[test]
    fn test_conf() {
        let model = DbModel::conf();
        println!("{:?}", model);
    }
}