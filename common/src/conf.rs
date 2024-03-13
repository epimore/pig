use std::fs::File;
use std::io::Read;
use anyhow::anyhow;
use clap::{Arg, Command};
use log::error;
use yaml_rust::{Yaml, YamlLoader};
use crate::err::{GlobalError, TransError};


pub fn make() -> Vec<Yaml> {
    let path = get_conf_path();
    let mut file = File::open(path).hand_err(|msg| error!("{msg}")).unwrap();
    let mut conf = String::new();
    file.read_to_string(&mut conf).hand_err(|msg| error!("{msg}")).unwrap();
    let vc = YamlLoader::load_from_str(&conf).hand_err(|msg| error!("{msg}")).unwrap();
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
        .hand_err(|msg| error!("{msg}"))
        .unwrap()
        .ok_or(GlobalError::SysErr(anyhow!("miss config path")))
        .hand_err(|msg| error!("{msg}"))
        .unwrap()
        .to_string()
}