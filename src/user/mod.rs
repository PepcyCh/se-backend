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
        comments::{Comment, NewComment},
        departments::DepartData,
        doctors::DoctorData,
        times::TimeData,
        user_logins::UserLoginData,
        users::UpdateUser,
    },
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{self, bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;

use self::{requests::*, responses::*, utils::get_username_from_token};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(view_info)
        .service(modify_password)
        .service(modify_info)
        .service(appoint)
        .service(cancel_appoint)
        .service(comment)
        .service(delete_comment)
        .service(search_depart)
        .service(search_doctor)
        .service(search_comment)
        .service(search_time)
        .service(search_appoint);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (view_info, "/view_info", ViewInfoRequest, ViewInfoResponse),
    (modify_password, "/modify_password", ModifyPasswordRequest, SimpleResponse),
    (modify_info, "/modify_info", ModifyInfoRequest, SimpleResponse),
    (appoint, "/appoint", AppointRequest, SimpleResponse),
    (cancel_appoint, "/cancel_appoint", CancelAppointRequest, SimpleResponse),
    (comment, "/comment", CommentRequest, SimpleResponse),
    (delete_comment, "/delete_comment", DeleteCommentRequest, SimpleResponse),
    (search_depart, "/search_depart", SearchDepartRequest, SearchDepartResponse),
    (search_doctor, "/search_doctor", SearchDoctorRequest, SearchDoctorResponse),
    (search_comment, "/search_comment", SearchCommentRequest, SearchCommentResponse),
    (search_time, "/search_time", SearchTimeRequest, SearchTimeResponse),
    (search_appoint, "/search_appoint", SearchAppointRequest, SearchAppointResponse),
}

async fn register_impl(
    pool: web::Data<DbPool>,
    info: web::Json<RegisterRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;

    web::block(move || {
        conn.transaction(|| {
            let res = users::table
                .filter(users::username.eq(&info.username))
                .count()
                .get_result::<i64>(&conn)
                .context("数据库错误")?;
            if res > 0 {
                bail!("用户名重复");
            }

            // TODO - gender check

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

            diesel::insert_into(users::table)
                .values(data)
                .execute(&conn)
                .context("数据库错误")?;

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
    use crate::schema::{user_logins, users};

    let info = info.into_inner();
    assert::assert_user(&pool, info.username.clone(), true).await?;

    let conn = get_db_conn(&pool)?;
    let login_token = web::block(move || {
        conn.transaction(|| {
            let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
            let res = users::table
                .filter(users::username.eq(&info.username))
                .filter(users::password.eq(&hashed_password))
                .filter(users::is_banned.eq(false))
                .count()
                .get_result::<i64>(&conn)
                .context("数据库错误")?;
            if res != 1 {
                bail!("密码错误")
            }

            let login_token = format!("{:x}", Blake2b::digest(info.username.as_bytes()));
            let token_data = UserLoginData {
                token: login_token.clone(),
                username: info.username,
                login_time: Utc::now().naive_utc(),
            };
            diesel::insert_into(user_logins::table)
                .values(token_data)
                .execute(&conn)
                .context("数据库错误")?;

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
    use crate::schema::user_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(user_logins::table.filter(user_logins::token.eq(info.login_token)))
            .execute(&conn)
    })
    .await
    .context("数据库错误")?;

    Ok(SimpleResponse::ok())
}

async fn view_info_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ViewInfoRequest>,
) -> anyhow::Result<ViewInfoResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let conn = get_db_conn(&pool)?;
    let res = users::table
        .filter(users::username.eq(&username))
        .get_result::<UserData>(&conn)
        .context("数据库错误")?;
        
    let data = ViewInfoResponse {
        success: true,
        err: "".to_string(),
        username: res.username,
        name: res.name,
        birthday: format!("{}", res.birthday.unwrap_or(NaiveDate::from_ymd(1970, 1, 1))),
        gender: res.gender,
        telephone: res.telephone,
    };
    Ok(data)
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
    web::block(move || {
        conn.transaction(|| {
            let hashed_password_old =
                format!("{:x}", Blake2b::digest(info.password_old.as_bytes()));
            let res = users::table
                .filter(users::username.eq(&username))
                .filter(users::password.eq(&hashed_password_old))
                .count()
                .get_result::<i64>(&conn)
                .context("数据库错误")?;
            if res != 1 {
                bail!("密码错误");
            }

            let hashed_password_new =
                format!("{:x}", Blake2b::digest(info.password_new.as_bytes()));
            diesel::update(users::table.filter(users::username.eq(&username)))
                .set(users::password.eq(&hashed_password_new))
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn modify_info_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyInfoRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let mut data = UpdateUser {
        name: info.name,
        gender: info.gender,
        telephone: info.telephone,
        ..Default::default()
    };
    if let Some(birthday) = info.birthday {
        let birthday = NaiveDate::parse_from_str(&birthday, "%Y-%m-%d").context("生日格式错误")?;
        data.birthday = Some(birthday);
    }

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::update(users::table.filter(users::username.eq(username)))
            .set(&data)
            .execute(&conn)
    })
    .await
    .context("数据库错误")?;

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
    web::block(move || {
        conn.transaction(|| {
            let res = appointments::table
                .filter(appointments::username.eq(&username))
                .filter(appointments::tid.eq(tid))
                .get_results::<Appointment>(&conn)
                .context("数据库错误")?;
            if res.len() > 0 && res[0].status != APPOINT_STATUS_CANCELED {
                bail!("预约已存在");
            }

            // check time
            let appo_time = times::table
                .filter(times::tid.eq(tid))
                .get_result::<TimeData>(&conn)
                .context("数据库错误")?;
            if appo_time.capacity <= appo_time.appointed {
                bail!("时间段已满");
            }

            // insert/update appo
            if res.is_empty() {
                let data = NewAppointment {
                    username,
                    tid,
                    status: APPOINT_STATUS_UNFINISHED.to_string(),
                    time: None,
                };
                diesel::insert_into(appointments::table)
                    .values(data)
                    .execute(&conn)
                    .context("数据库错误")?;
            } else {
                diesel::update(
                    appointments::table
                        .filter(appointments::username.eq(&username))
                        .filter(appointments::tid.eq(tid)),
                )
                .set(appointments::status.eq(APPOINT_STATUS_UNFINISHED))
                .execute(&conn)
                .context("数据库错误")?;
            }

            // update time
            diesel::update(times::table.filter(times::tid.eq(tid)))
                .set(times::appointed.eq(times::appointed + 1))
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

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
    web::block(move || {
        conn.transaction(|| {
            let res = appointments::table
                .filter(appointments::username.eq(&username))
                .filter(appointments::tid.eq(&tid))
                .get_results::<Appointment>(&conn)
                .context("数据库错误")?;
            if res.len() == 0 {
                bail!("预约不存在");
            }
            match res[0].status.as_str() {
                APPOINT_STATUS_FINISHED => bail!("预约已完成"),
                APPOINT_STATUS_CANCELED => bail!("预约已取消"),
                _ => {}
            }

            diesel::update(
                appointments::table
                    .filter(appointments::username.eq(&username))
                    .filter(appointments::tid.eq(tid)),
            )
            .set(appointments::status.eq(APPOINT_STATUS_CANCELED))
            .execute(&conn)
            .context("数据库错误")?;

            // update times
            diesel::update(times::table.filter(times::tid.eq(tid)))
                .set(times::appointed.eq(times::appointed - 1))
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

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
    .context("数据库错误")?;

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
    .context("数据库错误")?;

    Ok(SimpleResponse::ok())
}

async fn search_depart_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchDepartRequest>,
) -> anyhow::Result<SearchDepartResponse> {
    use crate::schema::departments;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username, true).await?;

    let conn = get_db_conn(&pool)?;
    let name_pattern = format!("%{}%", info.depart_name);
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let departs = web::block(move || {
        departments::table
            .filter(departments::depart_name.like(name_pattern))
            .order(departments::depart_name.asc())
            .offset(first_index)
            .limit(limit)
            .get_results::<DepartData>(&conn)
    })
    .await
    .context("数据库错误")?;

    let departs = departs
        .into_iter()
        .map(|data| SearchDepartItem {
            name: data.depart_name,
            info: data.information,
        })
        .collect();

    Ok(SearchDepartResponse {
        success: true,
        err: "".to_string(),
        departments: departs,
    })
}

async fn search_doctor_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchDoctorRequest>,
) -> anyhow::Result<SearchDoctorResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username, true).await?;

    let conn = get_db_conn(&pool)?;
    let depart_name_pattern = info
        .depart_name
        .map_or("%".to_string(), |s| format!("%{}%", s));
    let doctor_name_pattern = info
        .doctor_name
        .map_or("%".to_string(), |s| format!("%{}%", s));
    let rank = info.rank.unwrap_or("%".to_string());
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let docs = web::block(move || {
        doctors::table
            .filter(doctors::department.like(depart_name_pattern))
            .filter(doctors::name.like(doctor_name_pattern))
            .filter(doctors::rankk.like(rank))
            .order(doctors::name.asc())
            .offset(first_index)
            .limit(limit)
            .get_results::<DoctorData>(&conn)
    })
    .await
    .context("数据库错误")?;

    let docs = docs
        .into_iter()
        .map(|data| SearchDoctorItem {
            did: data.did,
            name: data.name,
            depart: data.department,
            rank: data.rank,
            gender: data.gender,
            age: data
                .birthday
                .map_or(-1, |birth| Utc::today().year() - birth.year()),
            info: data.information,
        })
        .collect();

    Ok(SearchDoctorResponse {
        success: true,
        err: "".to_string(),
        doctors: docs,
    })
}

async fn search_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchCommentRequest>,
) -> anyhow::Result<SearchCommentResponse> {
    use crate::schema::comments;
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S";

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username, true).await?;

    let time_min =
        NaiveDateTime::parse_from_str("1901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let time_max =
        NaiveDateTime::parse_from_str("2901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let start_time = info.start_time.map_or(Ok(time_min.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("起始时间格式错误")
    })?;
    let end_time = info.end_time.map_or(Ok(time_max.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("结束时间格式错误")
    })?;
    let did = info.did;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let cmts = web::block(move || {
        comments::table
            .filter(comments::did.eq(did))
            .filter(comments::time.between(start_time, end_time))
            .order(comments::time.desc())
            .offset(first_index)
            .limit(limit)
            .get_results::<Comment>(&conn)
    })
    .await
    .context("数据库错误")?;

    let cmts = cmts
        .into_iter()
        .map(|data| SearchCommentItem {
            cid: data.cid,
            username: data.username,
            comment: data.comment,
            time: format!("{}", data.time.unwrap().format(TIME_FMT)),
        })
        .collect();

    Ok(SearchCommentResponse {
        success: true,
        err: "".to_string(),
        comments: cmts,
    })
}

async fn search_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchTimeRequest>,
) -> anyhow::Result<SearchTimeResponse> {
    use crate::schema::times;
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S";

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username, true).await?;

    let time_min =
        NaiveDateTime::parse_from_str("1901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let time_max =
        NaiveDateTime::parse_from_str("2901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let start_time = info.start_time.map_or(Ok(time_min.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("起始时间格式错误")
    })?;
    let end_time = info.end_time.map_or(Ok(time_max.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("结束时间格式错误")
    })?;
    let did = info.did;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let show_all = info.show_all;
    let tms = web::block(move || {
        times::table
            .filter(times::did.eq(&did))
            .filter(times::start_time.ge(&start_time))
            .filter(times::end_time.le(&end_time))
            .filter((times::capacity.gt(times::appointed)).or(show_all))
            .order(times::start_time.asc())
            .offset(first_index)
            .limit(limit)
            .get_results::<TimeData>(&conn)
    })
    .await
    .context("数据库错误")?;

    let tms = tms
        .into_iter()
        .map(|data| SearchTimeItem {
            tid: data.tid,
            start_time: format!("{}", data.start_time.format(TIME_FMT)),
            end_time: format!("{}", data.end_time.format(TIME_FMT)),
            capacity: data.capacity,
            rest: data.capacity - data.appointed,
        })
        .collect();

    Ok(SearchTimeResponse {
        success: true,
        err: "".to_string(),
        times: tms,
    })
}

async fn search_appoint_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchAppointRequest>,
) -> anyhow::Result<SearchAppointResponse> {
    use crate::schema::{appointments, doctors, times};
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S";

    let info = info.into_inner();
    let username = get_username_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, username.clone(), true).await?;

    let time_min =
        NaiveDateTime::parse_from_str("1901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let time_max =
        NaiveDateTime::parse_from_str("2901-1-1T00:00:00", TIME_FMT).context("未知错误")?;
    let start_time = info.start_time.map_or(Ok(time_min.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("起始时间格式错误")
    })?;
    let end_time = info.end_time.map_or(Ok(time_max.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("结束时间格式错误")
    })?;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let status = info.status;
    let appos = web::block(move || {
        appointments::table
            .filter(appointments::username.eq(&username))
            .filter((appointments::status.eq(&status)).or(&status == "所有"))
            .inner_join(times::table.on(appointments::tid.eq(times::tid)))
            .filter(times::start_time.ge(&start_time))
            .filter(times::end_time.le(&end_time))
            .inner_join(doctors::table.on(times::did.eq(doctors::did)))
            .order(times::start_time.desc())
            .offset(first_index)
            .limit(limit)
            .get_results::<(Appointment, TimeData, DoctorData)>(&conn)
    })
    .await
    .context("数据库错误")?;

    let appos = appos
        .into_iter()
        .map(|(appo_data, time_data, doctor_data)| SearchAppointItem {
            did: doctor_data.did,
            doctor_name: doctor_data.name,
            start_time: format!("{}", time_data.start_time.format(TIME_FMT)),
            end_time: format!("{}", time_data.end_time.format(TIME_FMT)),
            status: appo_data.status,
            time: format!("{}", appo_data.time.unwrap().format(TIME_FMT)),
        })
        .collect();

    Ok(SearchAppointResponse {
        success: true,
        err: "".to_string(),
        appointments: appos,
    })
}
