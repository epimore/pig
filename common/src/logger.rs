use std::collections::HashMap;
use std::ops::Index;
use std::path::PathBuf;
use chrono::Local;
use log::LevelFilter;
use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Logger {
    store_path: PathBuf,
    suffix: String,
    level: String,
    //crate:level
    specify: Option<HashMap<String, LevelFilter>>,
}

impl Logger {
    pub fn init(vc: &Vec<Yaml>) -> Logger {
        let log = Logger::build(vc);
        let level: LevelFilter = Self::build_level_filter(&log.level);
        let path = std::path::Path::new(&log.store_path);
        std::fs::create_dir_all(path).unwrap();
        let mut dispatch = fern::Dispatch::new()
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
            })
            .level(level);
        if let Some(specify_map) = log.specify.clone() {
            for (k, v) in specify_map {
                dispatch = dispatch.level_for(k, v);
            }
        }
        dispatch.chain(std::io::stdout())
            .chain(fern::DateBased::new(&log.store_path, "%Y-%m-%d_".to_string() + &log.suffix + ".log"))
            .apply().expect("Logger initialization failed");
        log
    }

    fn build_level_filter(level: &str) -> LevelFilter {
        match level.to_uppercase().as_str() {
            "OFF" => LevelFilter::Off,
            "ERROR" => LevelFilter::Error,
            "WARN" => LevelFilter::Warn,
            "INFO" => LevelFilter::Info,
            "DEBUG" => LevelFilter::Debug,
            "TRACE" => LevelFilter::Trace,
            _ => panic!("The log level is invalid"),
        }
    }
    fn build(vc: &Vec<Yaml>) -> Self {
        let cfg = &vc[0];
        if cfg.is_badvalue() || cfg["log"].is_badvalue() {
            Logger {
                store_path: PathBuf::from("./logs/"),
                suffix: "server".to_string(),
                level: "error".to_string(),
                specify: None,
            }
        } else {
            let log = &cfg["log"];
            let sp = log["store_path"].as_str().unwrap_or("./logs/");
            let spn = if sp.ends_with("/") { sp.to_string() } else { format!("{}/", sp) };
            let mut specify = None;
            let mut specify_map = HashMap::new();
            if let Some(arr) = log["specify"].as_vec() {
                for item in arr {
                    let crate_str = item.index("crate").as_str().expect("指定log: 无crate名称").to_string();
                    let level_str = item.index("level").as_str().expect("指定log: 无level");
                    specify_map.insert(crate_str, Self::build_level_filter(level_str));
                }
                specify = Some(specify_map);
            }
            Logger {
                store_path: PathBuf::from(spn),
                suffix: (&log["suffix"].as_str()).unwrap_or("server").to_string(),
                level: (&log["level"].as_str()).unwrap_or("error").to_string(),
                specify,
            }
        }
    }
}