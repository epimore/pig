use std::path::PathBuf;
use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use yaml_rust::Yaml;
use cfg::{conf, Conf};
use serde::{Deserialize, Deserializer};

///
/// log:
//   level: debug #默认日志等级
//   prefix: server #默认日志文件前缀; eg:server_2024-10-26.log
//   store_path: ./logs
//   specify: #指定日志输出
//     - crate_name: hyper::http::h1  #或者hyper用指全部
//       level: Info #日志等级
//       file_name_prefix: hyper #日志文件前缀
//       additivity: false #是否记录到其父日志文件中
#[derive(Debug, Deserialize)]
#[conf(prefix = "log")]
pub struct Logger {
    store_path: PathBuf,
    prefix: String,
    #[serde(deserialize_with = "validate_level")]
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
                if let Some(prefix) = &s.file_name_prefix {
                    module_dispatch = module_dispatch
                        .chain(std::io::stdout())
                        .chain(fern::DateBased::new(&log.store_path, format!("{}_{}.log", prefix, "%Y-%m-%d".to_string())));
                }

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

#[cfg(test)]
mod tests {
    use cfg::{Conf};
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
}