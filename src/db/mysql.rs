use anyhow::anyhow;
use log::error;
use ::mysql::{OptsBuilder, Pool, PoolConstraints, PoolOpts,PooledConn};
use yaml_rust::Yaml;
use crate::err::{GlobalError, GlobalResult, TransError};
use crate::utils::crypto;
use super::*;

static mut POOL: Option<Pool> = Option::<Pool>::None;

pub fn get_conn() -> GlobalResult<PooledConn> {
    let res = unsafe { POOL.clone().ok_or(anyhow!("Mysql is not initialized"))?.get_conn().hand_err(|msg| { error!("{msg}") })? };
    Ok(res)
}

pub fn init_mysql(vc: &Vec<Yaml>) {
    let pool = build_pool(vc).expect("MySQL initialization failed");
    unsafe { POOL = Some(pool) };
}

fn build_pool(vc: &Vec<Yaml>) -> GlobalResult<Pool> {
    let db = DbModel::get_db_mode_by_type(vc, MYSQL).ok_or(GlobalError::SysErr(anyhow!("miss config path")))
        .hand_err(|msg| error!("{msg}"))?;
    let pm = DbPoolModel::build_pool_model_by_type(vc, MYSQL)?;
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
    let pool = Pool::new(opts).hand_err(|msg| error!("{msg}"))?;
    Ok(pool)
}