mod requests;
mod responses;
mod utils;

use crate::{
    database::{assert, get_db_conn},
    models::{admin_logins::AdminLoginData, administrators::AdminData, doctors::DoctorData},
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;

use self::{
    requests::{AddDoctorRequest, LoginRequest, LogoutRequest, RegisterRequest},
    responses::LoginResponse,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(add_doctor);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (add_doctor, "/add_doctor", AddDoctorRequest, SimpleResponse),
}

async fn register_impl(
    pool: web::Data<DbPool>,
    info: web::Json<RegisterRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::administrators;

    let info = info.into_inner();

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let res = administrators::table
                .filter(administrators::aid.eq(&info.aid))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res > 0 {
                bail!("duplicated ID");
            }

            let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
            let data = AdminData {
                aid: info.aid,
                password: hashed_password,
            };
            diesel::insert_into(administrators::table)
                .values(data)
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn login_impl(
    pool: web::Data<DbPool>,
    info: web::Json<LoginRequest>,
) -> anyhow::Result<LoginResponse> {
    use crate::schema::{admin_logins, administrators};

    let info = info.into_inner();
    assert::assert_admin(&pool, info.aid.clone()).await?;

    let conn = get_db_conn(&pool)?;
    let login_token = web::block(move || {
        conn.transaction(|| {
            let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
            let res = administrators::table
                .filter(administrators::aid.eq(&info.aid))
                .filter(administrators::password.eq(&hashed_password))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;

            if res != 1 {
                bail!("Wrong password");
            }

            let login_token = format!("{:x}", Blake2b::digest(info.aid.to_string().as_bytes()));
            let token_data = AdminLoginData {
                token: login_token.clone(),
                aid: info.aid,
                login_time: Utc::now().naive_utc(),
            };
            diesel::insert_into(admin_logins::table)
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
    use crate::schema::admin_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(admin_logins::table.filter(admin_logins::token.eq(info.login_token)))
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn add_doctor_impl(
    pool: web::Data<DbPool>,
    info: web::Json<AddDoctorRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    utils::get_aid_from_token(info.login_token.clone(), &pool).await?;
    assert::assert_depart(&pool, info.depart.clone()).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let res = doctors::table
                .filter(doctors::did.eq(&info.did))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res > 0 {
                bail!("duplicated ID");
            }

            // TODO - gender check

            let birthday = match NaiveDate::parse_from_str(&info.birthday, "%Y-%m-%d") {
                Ok(date) => Some(date),
                Err(_) => None,
            };
            // TODO - frontend hashed password ?
            let hashed_password = format!("{:x}", Blake2b::digest("123456".as_bytes()));
            let data = DoctorData {
                did: info.did,
                name: info.name,
                password: hashed_password,
                gender: info.gender,
                birthday,
                department: info.depart,
                rank: info.rank,
                infomation: None,
            };
            diesel::insert_into(doctors::table)
                .values(data)
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}
