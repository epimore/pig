use std::path::PathBuf;
use chrono::Local;
use log::LevelFilter;
use yaml_rust::Yaml;
use fern::colors::{Color, ColoredLevelConfig};

#[derive(Debug)]
pub struct Logger {
    store_path: PathBuf,
    suffix: String,
    level: String,
}

impl Logger {
    pub fn init(vc: &Vec<Yaml>) -> Logger {
        let log = Logger::build(vc);
        let level: LevelFilter = Self::build_level_filter(&log.level);
        let path = std::path::Path::new(&log.store_path);
        std::fs::create_dir_all(path).unwrap();
        let colors = ColoredLevelConfig::new()
            .info(Color::Green)
            .debug(Color::Blue)
            .error(Color::Red)
            .warn(Color::Yellow)
            .trace(Color::White);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] [{}] {} {}\n{}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    colors.color(record.level()),
                    record.target(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    message,
                ))
            })
            .level(level)
            .chain(std::io::stdout())
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
            }
        } else {
            let log = &cfg["log"];
            let sp = log["store_path"].as_str().unwrap_or("./logs/");
            let spn = if sp.ends_with("/") { sp.to_string() } else { format!("{}/", sp) };
            Logger {
                store_path: PathBuf::from(spn),
                suffix: (&log["suffix"].as_str()).unwrap_or("server").to_string(),
                level: (&log["level"].as_str()).unwrap_or("error").to_string(),
            }
        }
    }
}