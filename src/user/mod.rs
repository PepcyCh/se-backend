mod requests;
mod responses;
mod utils;

use crate::{
    database::{assert, get_db_conn},
    models::users::UserData,
    models::{
        appointments::{
            Appointment, NewAppointment, APPOINT_STATUS_CANCELED, APPOINT_STATUS_FINISHED,
            APPOINT_STATUS_UNFINISHED,
        },
        comments::NewComment,
        times::TimeData,
        user_logins::UserLoginData,
    },
    protocol::SimpleResponse,
    user::utils::get_username_from_token,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{self, bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;

use self::{
    requests::{
        AppointRequest, CancelAppointRequest, CommentRequest, DeleteCommentRequest, LoginRequest,
        LogoutRequest, ModifyPasswordRequest, RegisterRequest,
    },
    responses::LoginResponse,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(modify_password)
        .service(appoint)
        .service(cancel_appoint)
        .service(comment)
        .service(delete_comment);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (modify_password, "/modify_password", ModifyPasswordRequest, SimpleResponse),
    (appoint, "/appoint", AppointRequest, SimpleResponse),
    (cancel_appoint, "/cancel_appoint", CancelAppointRequest, SimpleResponse),
    (comment, "/comment", CommentRequest, SimpleResponse),
    (delete_comment, "/delete_comment", DeleteCommentRequest, SimpleResponse),
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
    assert::assert_user(&pool, username.clone(), true).await?;

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
        bail!("Wrong password")
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

async fn modify_password_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyPasswordRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token.clone(), &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let conn = get_db_conn(&pool)?;
    let username_temp = username.clone();
    let hashed_password_old = format!("{:x}", Blake2b::digest(info.password_old.as_bytes()));
    let res = web::block(move || {
        users::table
            .filter(users::username.eq(username_temp))
            .filter(users::password.eq(hashed_password_old))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("DB error")?;
    if res != 1 {
        bail!("Wrong password");
    }

    let conn = get_db_conn(&pool)?;
    let hashed_password_new = format!("{:x}", Blake2b::digest(info.password_new.as_bytes()));
    web::block(move || {
        diesel::update(users::table.filter(users::username.eq(username)))
            .set(users::password.eq(hashed_password_new))
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
    use crate::schema::{appointments, times};

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token.clone(), &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let tid = info.tid;
    assert::assert_time(&pool, tid).await?;

    // check appo
    let conn = get_db_conn(&pool)?;
    let username_temp = username.clone();
    let res = web::block(move || {
        appointments::table
            .filter(appointments::username.eq(username_temp))
            .filter(appointments::tid.eq(tid))
            .get_results::<Appointment>(&conn)
    })
    .await
    .context("DB error")?;
    if res.len() > 0 && res[0].status != APPOINT_STATUS_CANCELED {
        bail!("Appointment already exists");
    }

    // check time
    let conn = get_db_conn(&pool)?;
    let appo_time = web::block(move || {
        times::table
            .filter(times::tid.eq(tid))
            .get_result::<TimeData>(&conn)
    })
    .await
    .context("DB error")?;

    if appo_time.rest == 0 {
        bail!("There is no space in thie time");
    }

    // insert/update appo
    let conn = get_db_conn(&pool)?;
    if res.is_empty() {
        let data = NewAppointment {
            username,
            tid,
            status: APPOINT_STATUS_UNFINISHED.to_string(),
            time: None,
        };
        web::block(move || {
            diesel::insert_into(appointments::table)
                .values(data)
                .execute(&conn)
        })
        .await
        .context("DB error")?;
    } else {
        web::block(move || {
            diesel::update(
                appointments::table
                    .filter(appointments::username.eq(username))
                    .filter(appointments::tid.eq(tid)),
            )
            .set(appointments::status.eq(APPOINT_STATUS_UNFINISHED))
            .execute(&conn)
        })
        .await
        .context("DB error")?;
    }

    // update time
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::update(times::table.filter(times::tid.eq(tid)))
            .set(times::rest.eq(times::rest - 1))
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn cancel_appoint_impl(
    pool: web::Data<DbPool>,
    info: web::Json<CancelAppointRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::{appointments, times};

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token.clone(), &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let tid = info.tid;
    assert::assert_time(&pool, tid).await?;

    let conn = get_db_conn(&pool)?;
    let username_temp = username.clone();
    let res = web::block(move || {
        appointments::table
            .filter(appointments::username.eq(username_temp))
            .filter(appointments::tid.eq(tid))
            .get_results::<Appointment>(&conn)
    })
    .await
    .context("DB error")?;
    if res.len() == 0 {
        bail!("Appointment doesn't exist");
    }
    match res[0].status.as_str() {
        APPOINT_STATUS_FINISHED => bail!("Appointment has been finished"),
        APPOINT_STATUS_CANCELED => bail!("Appointment has been canceled"),
        _ => {}
    }

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::update(
            appointments::table
                .filter(appointments::username.eq(username))
                .filter(appointments::tid.eq(tid)),
        )
        .set(appointments::status.eq(APPOINT_STATUS_CANCELED))
        .execute(&conn)
    })
    .await
    .context("DB error")?;

    // update times
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::update(times::table.filter(times::tid.eq(tid)))
            .set(times::rest.eq(times::rest + 1))
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<CommentRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::comments;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;

    assert::assert_user(&pool, username.clone(), true).await?;
    assert::assert_doctor(&pool, info.did.clone()).await?;

    let conn = get_db_conn(&pool)?;
    let data = NewComment {
        username,
        did: info.did,
        comment: info.comment,
    };
    web::block(move || {
        diesel::insert_into(comments::table)
            .values(data)
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn delete_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<DeleteCommentRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::comments;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;

    assert::assert_user(&pool, username.clone(), true).await?;
    assert::assert_comment(&pool, info.cid).await?;

    let conn = get_db_conn(&pool)?;
    let cid = info.cid;
    web::block(move || {
        diesel::delete(comments::table.filter(comments::cid.eq(cid))).execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}
