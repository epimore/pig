use yaml_rust::Yaml;

pub mod conf;
pub mod err;
mod log;
mod db;
mod utils;
pub mod net;

///just build config info and log;
pub fn init() -> Tripe {
    Tripe::new()
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Tripe {
    cfg: Vec<Yaml>,
    logger: log::Logger,
    enable_mysql: bool,
}

impl Tripe {
    fn new() -> Self {
        let vc = conf::make();
        let logger = log::Logger::init(&vc);
        Self {
            cfg: vc,
            logger,
            enable_mysql: false,
        }
    }
    #[cfg(feature = "db-mysql")]
    pub fn enable_mysql(mut self) -> Self {
        self.enable_mysql = true;
        db::mysql::init_mysql(&self.cfg);
        self
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
