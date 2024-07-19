use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use anyhow::anyhow;
use clap::{Arg, Command};
use log::error;
use once_cell::sync::Lazy;
use yaml_rust::{Yaml, YamlLoader};
use crate::err::{GlobalError, TransError};

static CONF: Lazy<Arc<Vec<Yaml>>> = Lazy::new(|| Arc::new(make()));

pub fn get_config() -> Arc<Vec<Yaml>> {
    CONF.clone()
}

fn make() -> Vec<Yaml> {
    let path = get_conf_path();
    // let path = "/home/ubuntu20/code/rs/mv/github/epimore/gmv/stream/config.yml".to_string();
    // let path = "/home/ubuntu20/code/rs/mv/github/epimore/gmv/session/config.yml".to_string();
    let mut file = File::open(path).hand_log(|msg| error!("{msg}")).unwrap();
    let mut conf = String::new();
    file.read_to_string(&mut conf).hand_log(|msg| error!("{msg}")).unwrap();
    let vc = YamlLoader::load_from_str(&conf).hand_log(|msg| error!("{msg}")).unwrap();
    vc
}

fn get_conf_path() -> String {
    let matches = Command::new("MyApp")
        .version("1.0")
        .author("Kz. <kz986542@gmail.com>")
        .about("get the path about config file")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Path to configuration file")
            .default_value("./config.yml")
        )
        .get_matches();
    matches.try_get_one::<String>("config")
        .hand_log(|msg| error!("{msg}"))
        .unwrap()
        .ok_or(GlobalError::SysErr(anyhow!("miss config path")))
        .hand_log(|msg| error!("{msg}"))
        .unwrap()
        .to_string()
}