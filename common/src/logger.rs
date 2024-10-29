use std::path::PathBuf;

use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use serde::{Deserialize, Deserializer};

use cfg_lib::{conf, Conf};

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
    #[serde(default = "default_app")]
    prefix: String,
    #[serde(deserialize_with = "validate_level")]
    #[serde(default = "default_info")]
    level: String,
    specify: Option<Vec<Specify>>,
}

#[derive(Debug, Deserialize)]
pub struct Specify {
    crate_name: String,
    #[serde(deserialize_with = "validate_level")]
    level: String,
    file_name_prefix: Option<String>,
    additivity: Option<bool>,
}


impl Logger {
    pub fn init() -> Logger {
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
        log
    }
}

fn level_filter(level: &str) -> LevelFilter {
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

fn validate_level<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let level = String::deserialize(deserializer)?;
    level_filter(&*level);
    Ok(level)
}

fn default_info() -> String {
    "info".to_string()
}

fn default_app() -> String {
    "app".to_string()
}

#[cfg(test)]
mod tests {
    use cfg_lib::Conf;

    use super::*;

    #[test]
    fn test_conf_log() {
        let logger = Logger::conf();
        println!("{:?}", logger);
    }

    #[test]
    fn test_log() {
        let _logger = Logger::init();

        log::info!("This is a default log message.");
        log::debug!("This is a debug log message.");
        log::error!("This is an error log message.");

        // 指定模块日志
        log::info!(target: "hyper::http::h1", "This is a hyper::http::h1 log message.");
        log::debug!(target: "hyper", "This is a hyper debug log message.");
    }

    #[test]
    fn test_default_value() {
        println!("{:?}", PathBuf::default());
    }
}