use common::anyhow::anyhow;
use common::log::error;
use ::mysql::{OptsBuilder, Pool, PoolConstraints, PoolOpts,PooledConn};
use common::yaml_rust::Yaml;
use common::err::{GlobalError, GlobalResult, TransError};
use common::utils::crypto;
use super::*;

static mut POOL: Option<Pool> = Option::<Pool>::None;

pub fn get_mysql_conn() -> GlobalResult<PooledConn> {
    let res = unsafe { POOL.clone().ok_or(anyhow!("Mysql is not initialized"))?.get_conn().hand_log(|msg| { error!("{msg}") })? };
    Ok(res)
}

pub fn init_mysql(cfg: &Yaml) {
    let pool = build_pool(cfg).expect("MySQL initialization failed");
    unsafe { POOL = Some(pool) };
}

fn build_pool(cfg: &Yaml) -> GlobalResult<Pool> {
    let db = DbModel::get_db_mode_by_type(cfg, MYSQL).ok_or(GlobalError::SysErr(anyhow!("miss config path")))
        .hand_log(|msg| error!("{msg}"))?;
    let pm = DbPoolModel::build_pool_model_by_type(cfg, MYSQL)?;
    let pool_opts = PoolConstraints::new(pm.min_size, pm.max_size)
        .map(
            |pc|
                PoolOpts::new()
                    .with_constraints(pc)
                    .with_check_health(pm.check_health));
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(db.host_or_ip))
        .tcp_port(db.port)
        .db_name(Some(db.db_name))
        .user(db.user)
        .tcp_connect_timeout(Some(pm.connection_timeout))
        .read_timeout(pm.read_timeout)
        .write_timeout(pm.write_timeout)
        .pool_opts(pool_opts)
        .pass(db.pass.map(|data| crypto::default_decrypt(&data).expect("MYSQL密码错误")));
    let pool = Pool::new(opts).hand_log(|msg| error!("{msg}"))?;
    Ok(pool)
}