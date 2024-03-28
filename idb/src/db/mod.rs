use std::collections::HashMap;
use std::time::Duration;
use common::anyhow::anyhow;
use common::yaml_rust::Yaml;
use common::err::GlobalResult;
#[allow(dead_code)]
pub const MYSQL: &str = "mysql";
pub mod imysql;

#[derive(Debug)]
pub struct DbModel {
    pub host_or_ip: String,
    pub port: u16,
    pub db_name: String,
    pub user: Option<String>,
    pub pass: Option<String>,
    //暂时不使用
    pub connect_attrs: Option<HashMap<String, String>>,
}
#[allow(dead_code)]
impl DbModel {
    //db_type  =  [redis,mysql]
    pub fn get_db_mode_by_type(cfg: &Yaml, db_type: &str) -> Option<DbModel> {
        if !cfg.is_badvalue() && !cfg["db"].is_badvalue() {
            let dbs = &cfg["db"];
            let dbi = &dbs[db_type];
            if !dbi.is_badvalue() {
                return Some(DbModel {
                    host_or_ip: dbi["host_or_ip"].as_str().expect("database must give a host or ipV4").to_string(),
                    port: dbi["port"].as_i64().expect("database must give a port") as u16,
                    db_name: dbi["db_name"].as_str().expect("database must give a name").to_string(),
                    user: dbi["user"].clone().into_string(),
                    pass: dbi["pass"].clone().into_string(),
                    connect_attrs: None,
                });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct DbPoolModel {
    pub max_size: usize,
    pub min_size: usize,
    pub check_health: bool,
    pub read_timeout: Option<Duration>,
    pub write_timeout: Option<Duration>,
    pub connection_timeout: Duration,
}
#[allow(dead_code)]
impl DbPoolModel {
    //db_type  =  [redis,mysql]
    pub fn build_pool_model_by_type(cfg: &Yaml, db_type: &str) -> GlobalResult<DbPoolModel> {
        if !cfg.is_badvalue() && !cfg["db"].is_badvalue() {
            let dbs = &cfg["db"];
            let dbi = &dbs[db_type];
            if !dbi.is_badvalue() {
                let dbp = &dbi["pool"];
                if !dbp.is_badvalue() {
                    let max_size = if dbp["max_size"].is_badvalue() { 100 } else { dbp["max_size"].as_i64().ok_or(anyhow!("mysql config param [max_size] is invalid"))? as usize };
                    let min_size = if dbp["min_size"].is_badvalue() { max_size } else { dbp["min_size"].as_i64().ok_or(anyhow!("mysql config param [min_size] is invalid"))? as usize };
                    let check_health = if dbp["check_health"].is_badvalue() { true } else { dbp["check_health"].as_bool().ok_or(anyhow!("mysql config param [check_health] is invalid"))? };
                    let read_timeout = if dbp["read_timeout"].is_badvalue() {
                        None
                    } else {
                        Some(Duration::from_secs(dbp["read_timeout"].as_i64().ok_or(anyhow!("mysql config param [read_timeout] is invalid"))? as u64))
                    };
                    let write_timeout = if dbp["write_timeout"].is_badvalue() {
                        None
                    } else {
                        Some(Duration::from_secs(dbp["write_timeout"].as_i64().ok_or(anyhow!("mysql config param [write_timeout] is invalid"))? as u64))
                    };
                    let connection_timeout = if dbp["connection_timeout"].is_badvalue() {
                        Duration::from_secs(30)
                    } else {
                        Duration::from_secs(dbp["connection_timeout"].as_i64().ok_or(anyhow!("mysql config param [write_timeout] is invalid"))? as u64)
                    };

                    return Ok(DbPoolModel {
                        max_size,
                        min_size,
                        check_health,
                        read_timeout,
                        write_timeout,
                        connection_timeout,
                    });
                }
            }
        }
        Ok(DbPoolModel {
            max_size: 100,
            min_size: 100,
            check_health: true,
            read_timeout: None,
            write_timeout: None,
            connection_timeout: Duration::from_secs(30),
        })
    }
}
