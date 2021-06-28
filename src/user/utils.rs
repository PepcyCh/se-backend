use actix_web::web;
use anyhow::{bail, Context};
use chrono::Utc;
use diesel::prelude::*;

use crate::{models::user_logins::UserLoginData, DbPool};

pub async fn get_username_from_token(
    token: String,
    pool: &web::Data<DbPool>,
) -> anyhow::Result<String> {
    use crate::schema::user_logins;
    const MAX_LOGIN_TIME_SECS: i64 = 3600;

    let conn = pool.get().context("数据库连接错误")?;
    let data = web::block(move || {
        user_logins::table
            .filter(user_logins::token.eq(token))
            .order(user_logins::login_time.desc())
            .limit(1)
            .get_result::<UserLoginData>(&conn)
            .optional()
    })
    .await
    .context("数据库错误")?;

    if let Some(data) = data {
        let time_diff = Utc::now()
            .naive_utc()
            .signed_duration_since(data.login_time);
        if time_diff.num_seconds() <= MAX_LOGIN_TIME_SECS {
            return Ok(data.username);
        } else {
            bail!("登录已过期");
        }
    } else {
        bail!("您还未登录");
    }
}
