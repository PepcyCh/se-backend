pub mod assert;

use crate::DbPool;
use actix_web::web;
use anyhow::Context;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::PooledConnection;

pub fn get_db_conn(
    pool: &web::Data<DbPool>,
) -> anyhow::Result<PooledConnection<ConnectionManager<MysqlConnection>>> {
    pool.get().context("DB connection")
}
