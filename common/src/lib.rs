pub mod logger;
pub mod conf;
pub mod err;
#[cfg(feature = "net")]
pub mod net;
pub mod utils;

pub use tokio;
pub use dashmap;
pub use yaml_rust;
pub use log;
pub use anyhow;
pub use thiserror;
pub use bytes;
pub use fern;
pub use clap;
pub use chrono;
pub use once_cell;
pub use rand;
use constructor::Get;

///just build config info and log;
pub fn init() -> Tripe {
    Tripe::new()
}

#[derive(Debug, Get)]
#[allow(dead_code)]
pub struct Tripe {
    cfg: Vec<yaml_rust::Yaml>,
    logger: logger::Logger,
}

impl Tripe {
    fn new() -> Self {
        let vc = conf::make();
        let logger = logger::Logger::init(&vc);
        Self {
            cfg: vc,
            logger,
        }
    }
}
