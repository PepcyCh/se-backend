mod requests;
mod responses;
mod utils;

use crate::{
    database::{assert, get_db_conn},
    models::{doctor_logins::DoctorLoginData, times::NewTime},
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use self::{
    requests::{AddTimeRequest, LoginRequest, LogoutRequest},
    responses::LoginResponse,
    utils::get_did_from_token,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(logout).service(add_time);
}

crate::post_funcs! {
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (add_time, "/add_time", AddTimeRequest, SimpleResponse),
}

async fn login_impl(
    pool: web::Data<DbPool>,
    info: web::Json<LoginRequest>,
) -> anyhow::Result<LoginResponse> {
    use crate::schema::{doctor_logins, doctors};

    let info = info.into_inner();
    assert::assert_doctor(&pool, info.did.clone()).await?;

    let conn = get_db_conn(&pool)?;
    let login_token = web::block(move || {
        conn.transaction(|| {
            let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
            let res = doctors::table
                .filter(doctors::did.eq(&info.did))
                .filter(doctors::password.eq(hashed_password))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res != 1 {
                bail!("Wrong password");
            }

            let login_token = format!("{:x}", Blake2b::digest(info.did.to_string().as_bytes()));
            let token_data = DoctorLoginData {
                token: login_token.clone(),
                did: info.did,
                login_time: Utc::now().naive_utc(),
            };
            diesel::insert_into(doctor_logins::table)
                .values(token_data)
                .execute(&conn)
                .context("DB error")?;

            Ok(login_token)
        })
    })
    .await?;

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
    use crate::schema::doctor_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(doctor_logins::table.filter(doctor_logins::token.eq(info.login_token)))
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn add_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<AddTimeRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::times;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token.clone(), &pool).await?;

    let start_time = NaiveDateTime::parse_from_str(&info.start_time, "%Y-%m-%dT%H:%M:%S")
        .context("Wrong format on 'start_time'")?;
    let end_time = NaiveDateTime::parse_from_str(&info.end_time, "%Y-%m-%dT%H:%M:%S")
        .context("Wrong format on 'end_time'")?;
    if start_time >= end_time {
        bail!("Invalid time interval");
    }

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let res = times::table
                .filter(times::did.eq(&did))
                .filter(
                    times::start_time
                        .between(&start_time, &end_time)
                        .or(times::end_time.between(&start_time, &end_time)),
                )
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res > 0 {
                bail!("Time interval conflicts with existed times");
            }

            let data = NewTime {
                did,
                start_time,
                end_time,
                capacity: info.capacity,
            };
            diesel::insert_into(times::table)
                .values(data)
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}
