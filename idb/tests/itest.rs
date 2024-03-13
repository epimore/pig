use mysql::serde::{Deserialize, Serialize};
use constructor::{Get, New, Set};
use ezsql::crud;
use common::log::error;
use common::err::TransError;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Get, Set, New)]
#[new(device_id, domain_id, domain)]
#[crud(table_name = "GMV_OAUTH",
funs = [
{fn_name = "add_gmv_oauth", sql_type = "create:single", exist_update = "true"},
{fn_name = "add_gmv_oauth_by_batch", sql_type = "create:batch"},
{fn_name = "delete_gmv_oauth", sql_type = "delete", condition = "device_id:=,domain_id:="},
{fn_name = "update_gmv_oauth_pwd", sql_type = "update", fields = "pwd", condition = "device_id:=,domain_id:="},
{fn_name = "read_gmv_oauth_single_all", sql_type = "read:single", condition = "device_id:=,domain_id:="},
{fn_name = "read_gmv_oauth_single_pwd", sql_type = "read:single", fields = "pwd", condition = "device_id:=,domain_id:=", res_type = "false"},
{fn_name = "read_gmv_oauth_batch_status", sql_type = "read:batch", condition = "status:=", order = "device_id:desc" page = "true", res_type = "true"},
])]
struct GmvOauth {
    device_id: String,
    domain_id: String,
    domain: String,
    pwd: Option<String>,
    pwd_check: u8,
    alias: Option<String>,
    status: u8,
}

#[test]
fn test() {
    let tripe = common::init();
    idb::init_mysql(tripe.get_cfg());
    let mut conn = idb::get_mysql_conn().unwrap();

    let gmv_oauth = GmvOauth::new("device_id_1".to_string(), "domain_id_1".to_string(), "domain_1".to_string());
    gmv_oauth.add_gmv_oauth(&mut conn);

    let mut vec = Vec::new();
    for i in 0..10 {
        let gmv_oauth = GmvOauth::new(format!("batch_device_id_{}", i), format!("batch_domain_id_{}", i), format!("domain_{}", i));
        vec.push(gmv_oauth);
    };
    GmvOauth::add_gmv_oauth_by_batch(vec, &mut conn);

    GmvOauth::delete_gmv_oauth(&mut conn, format!("batch_device_id_{}", 1), format!("batch_domain_id_{}", 1));

    let mut update_oauth = GmvOauth::default();
    update_oauth.set_pwd(Some("123aaa".to_string()));
    update_oauth.update_gmv_oauth_pwd(&mut conn, format!("batch_device_id_{}", 2), format!("batch_domain_id_{}", 2));

    let all = GmvOauth::read_gmv_oauth_single_all(&mut conn, format!("batch_device_id_{}", 2), format!("batch_domain_id_{}", 2));
    log::info!("{all:?}");
    let pwd: Option<String> = GmvOauth::read_gmv_oauth_single_pwd(&mut conn, format!("batch_device_id_{}", 2), format!("batch_domain_id_{}", 2))
        .hand_err(|msg| error!("{msg}")).unwrap();
    log::info!("{pwd:?}");

    let page_limit = GmvOauth::read_gmv_oauth_batch_status(&mut conn, 0, 0, 5);
    log::info!("{page_limit:?}");
}