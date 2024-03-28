use mysql::PooledConn;
use common::yaml_rust::Yaml;
use common::err::GlobalResult;

mod db;

pub fn init_mysql(cfg: &Yaml) {
    db::imysql::init_mysql(cfg);
}

pub fn get_mysql_conn() -> GlobalResult<PooledConn> {
    db::imysql::get_mysql_conn()
}


