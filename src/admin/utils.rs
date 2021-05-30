use actix_web::web;
use anyhow::{bail, Context};
use chrono::Utc;
use diesel::prelude::*;

use crate::{models::admin_logins::AdminLoginData, DbPool};

pub async fn get_aid_from_token(token: String, pool: &web::Data<DbPool>) -> anyhow::Result<String> {
    use crate::schema::admin_logins;
    const MAX_LOGIN_TIME_SECS: i64 = 3600;

    let conn = pool.get().context("DB connection error")?;
    let data = web::block(move || {
        admin_logins::table
            .filter(admin_logins::token.eq(token))
            .order(admin_logins::login_time.desc())
            .limit(1)
            .get_result::<AdminLoginData>(&conn)
            .optional()
    })
    .await
    .context("DB error")?;

    if let Some(data) = data {
        let time_diff = Utc::now()
            .naive_utc()
            .signed_duration_since(data.login_time);
        if time_diff.num_seconds() <= MAX_LOGIN_TIME_SECS {
            return Ok(data.aid);
        } else {
            bail!("Login has expired");
        }
    } else {
        bail!("No such login token");
    }
}
