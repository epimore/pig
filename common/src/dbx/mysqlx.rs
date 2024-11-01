use std::time::Duration;

use log::{error, LevelFilter};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use sqlx::{Connection, ConnectOptions, MySql, Pool};
use sqlx::mysql::MySqlSslMode;
use sqlx::pool::PoolOptions;

use cfg_lib::{conf};
use exception::{GlobalError, GlobalResult, TransError};

use crate::{logger, serde_default};
use crate::utils::crypto::{default_decrypt, default_encrypt};

static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();


#[cfg(feature = "mysqlx")]
pub fn init_conn_pool() {
    let pool_conn = DbModel::build_pool_conn();
    MYSQL_POOL.set(pool_conn).expect("Initializing mysql connection pool failed due to multiple settings");
}

#[cfg(feature = "mysqlx")]
pub fn get_conn_by_pool() -> GlobalResult<&'static Pool<MySql>> {
    let conn_pool = MYSQL_POOL.get().ok_or_else(|| GlobalError::new_sys_error("the mysql connection pool has not been initialized", |msg| error!("{msg}")))?;
    Ok(conn_pool)
}

#[derive(Debug, Deserialize)]
#[conf(
    prefix = "db.mysql",
    path = "/home/ubuntu20/code/rs/mv/github/epimore/pig/common/config.yml"
)]
struct DbModel {
    host_or_ip: String,
    port: u16,
    db_name: String,
    user: String,
    pass: String,
    attrs: Option<AttrsModel>,
    #[serde(default = "default_pool_model")]
    pool: PoolModel,
}
serde_default!(default_pool_model, PoolModel, PoolModel::default());
impl DbModel {
    fn build_pool_conn() -> Pool<MySql> {
        let model: DbModel = DbModel::conf();
        let mut conn_options = <<MySql as sqlx::Database>::Connection as Connection>::Options::new()
            .host(&*model.host_or_ip)
            .port(model.port)
            .database(&*model.db_name)
            .username(&*model.user)
            .password(&*default_decrypt(&*model.pass).expect("mysql pass invalid"));
        if let Some(attr) = model.attrs {
            if let Some(log) = attr.log_global_sql_level {
                let level = logger::level_filter(&*log);
                conn_options = conn_options.log_statements(level);
            }
            if let Some(timeout) = attr.log_slow_sql_timeout {
                conn_options = conn_options.log_slow_statements(LevelFilter::Warn, Duration::from_secs(timeout as u64));
            }
            if let Some(timezone) = attr.timezone {
                conn_options = conn_options.timezone(Some(timezone));
            }
            if let Some(charset) = attr.charset {
                conn_options = conn_options.charset(&*charset);
            }
            match attr.ssl_level {
                None | Some(1) => {}
                Some(0) => { conn_options = conn_options.ssl_mode(MySqlSslMode::Disabled); }
                Some(2) => { conn_options = conn_options.ssl_mode(MySqlSslMode::Required); }
                Some(3) => { conn_options = conn_options.ssl_mode(MySqlSslMode::VerifyIdentity); }
                Some(4) => {
                    conn_options = conn_options.ssl_mode(MySqlSslMode::VerifyCa);
                }
                Some(other) => { panic!("连接无效加密等级") }
            }
            if let Some(ca) = attr.ssl_ca_crt_file {
                conn_options = conn_options.ssl_ca(ca)
            }
            if let Some(cert) = attr.ssl_ca_client_cert_file {
                conn_options = conn_options.ssl_client_cert(cert);
            }
            if let Some(key) = attr.ssl_ca_client_key_file {
                conn_options = conn_options.ssl_client_key(key);
            }
        }
        model.pool.build_pool_options().connect_lazy_with(conn_options)
    }
}

#[derive(Debug, Deserialize)]
struct AttrsModel {
    log_global_sql_level: Option<String>,
    log_slow_sql_timeout: Option<u16>,
    timezone: Option<String>,
    charset: Option<String>,
    ssl_level: Option<u8>,
    ssl_ca_crt_file: Option<String>,
    ssl_ca_client_cert_file: Option<String>,
    ssl_ca_client_key_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PoolModel {
    #[serde(default = "default_max_connections")]
    max_connections: u32,
    #[serde(default = "default_min_connections")]
    min_connections: u32,
    #[serde(default = "default_connection_timeout")]
    connection_timeout: u8,
    #[serde(default = "default_max_lifetime")]
    max_lifetime: u32,
    #[serde(default = "default_idle_timeout")]
    idle_timeout: u32,
    #[serde(default = "default_check_health")]
    check_health: bool,
}

impl PoolModel {
    fn build_pool_options(self) -> PoolOptions<MySql> {
        PoolOptions::<MySql>::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.connection_timeout as u64))
            .max_lifetime(Duration::from_secs(self.max_lifetime as u64))
            .idle_timeout(Duration::from_secs(self.idle_timeout as u64))
            .test_before_acquire(self.check_health)
    }
}
serde_default!(default_max_connections, u32, DEFAULT_MAX_CONNECTIONS);
serde_default!(default_min_connections, u32, DEFAULT_MIN_CONNECTIONS);
serde_default!(default_connection_timeout, u8, DEFAULT_CONNECTION_TIMEOUT);
serde_default!(default_max_lifetime, u32, DEFAULT_MAX_LIFETIME);
serde_default!(default_idle_timeout, u32, DEFAULT_IDLE_TIMEOUT);
serde_default!(default_check_health, bool, DEFAULT_CHECK_HEALTH);
const DEFAULT_MAX_CONNECTIONS: u32 = 100;
const DEFAULT_MIN_CONNECTIONS: u32 = 100;
const DEFAULT_CONNECTION_TIMEOUT: u8 = 8;
const DEFAULT_MAX_LIFETIME: u32 = 30;
const DEFAULT_IDLE_TIMEOUT: u32 = 8;
const DEFAULT_CHECK_HEALTH: bool = true;

impl Default for PoolModel {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            check_health: DEFAULT_CHECK_HEALTH,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_conf() {
        let conf1 = DbModel::conf();
        println!("{:?}", conf1);
    }

    //cargo test --features mysqlx --package common --lib dbx::mysqlx::tests::test_mysql_query -- --exact --nocapture
    #[tokio::test]
    #[cfg(feature = "mysqlx")]
    async fn test_mysql_query() {
        logger::Logger::init();
        init_conn_pool();
        let pool = get_conn_by_pool().expect("获取连接失败");

        let x = sqlx::query("select * from GMV_OAUTH").fetch_one(pool).await;
        println!("res = {:?}", x);
    }
}