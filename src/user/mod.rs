mod requests;
mod responses;
mod utils;

use crate::{DbPool, database::{assert, get_db_conn}, models::{appointments::Appointment, user_logins::UserLoginData}, models::users::UserData, protocol::SimpleResponse, user::utils::get_username_from_token};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{self, bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;

use self::{requests::{AppointRequest, LoginRequest, LogoutRequest, RegisterRequest}, responses::LoginResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login).service(logout).service(appoint);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (appoint, "/appoint", AppointRequest, SimpleResponse),
}

async fn register_impl(
    pool: web::Data<DbPool>,
    info: web::Json<RegisterRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;

    let username = info.username.clone();
    let res = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("DB error")?;
    if res > 0 {
        bail!("duplicated username");
    }

    // TODO - gender check
    // if info.gender != "Male" && info.gender != "Female" {
    //     bail!("wrong gender")
    // }

    let birthday = match NaiveDate::parse_from_str(&info.birthday, "%Y-%m-%d") {
        Ok(date) => Some(date),
        Err(_) => None,
    };

    let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
    let data = UserData {
        username: info.username,
        password: hashed_password,
        name: info.name,
        gender: info.gender,
        birthday,
        telephone: info.telephone,
        is_banned: false,
    };

    let conn = pool.get().context("DB connection error")?;
    web::block(move || {
        diesel::insert_into(users::table)
            .values(data)
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn login_impl(
    pool: web::Data<DbPool>,
    info: web::Json<LoginRequest>,
) -> anyhow::Result<LoginResponse> {
    use crate::schema::{user_logins, users};

    let info = info.into_inner();

    let username = info.username.clone();
    let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
    let conn = get_db_conn(&pool)?;
    let res = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .filter(users::password.eq(&hashed_password))
            .filter(users::is_banned.eq(false))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("DB error")?;

    if res != 1 {
        bail!("Wrong username/Wrong password/User is banned")
    }

    let login_token = format!("{:x}", Blake2b::digest(info.username.as_bytes()));

    let token_data = UserLoginData {
        token: login_token.clone(),
        username: info.username.clone(),
        login_time: Utc::now().naive_utc(),
    };
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::insert_into(user_logins::table)
            .values(token_data)
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(LoginResponse {
        success: true,
        err: "".to_string(),
        login_token,
    })
}

async fn logout_impl(
    pool: web::Data<DbPool>,
    info: web::Json<LogoutRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::user_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(user_logins::table.filter(user_logins::token.eq(info.login_token)))
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn appoint_impl(
    pool: web::Data<DbPool>,
    info: web::Json<AppointRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::appointments;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token.clone(), &pool).await?;
    
    let tid = info.tid;
    assert::assert_time(&pool, tid).await?;

    let conn = get_db_conn(&pool)?;
    let username_temp = username.clone();
    let res = web::block(move || {
        appointments::table
            .filter(appointments::username.eq(username_temp))
            .filter(appointments::tid.eq(tid))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("DB error")?;
    if res > 0 {
        bail!("Appointment already exists");
    }

    let conn = get_db_conn(&pool)?;
    let data = Appointment {
        username,
        tid,
        status: "Unfinished".to_string(), // TODO - appoint - status str
        time: None,
    };
    web::block(move || {
        diesel::insert_into(appointments::table)
            .values(data)
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}