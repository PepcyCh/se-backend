mod requests;
mod responses;
mod utils;

use crate::{
    database::{assert, get_db_conn},
    models::{
        appointments::{Appointment, APPOINT_STATUS_FINISHED, APPOINT_STATUS_UNFINISHED},
        comments::Comment,
        doctor_logins::DoctorLoginData,
        doctors::{DoctorData, UpdateDoctor},
        times::{NewTime, TimeData, UpdateTime},
        users::UserData,
    },
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;

use self::{requests::*, responses::*, utils::get_did_from_token};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(logout)
        .service(view_info)
        .service(modify_password)
        .service(modify_info)
        .service(add_time)
        .service(modify_time)
        .service(delete_time)
        .service(search_time)
        .service(search_appoint)
        .service(finish_appoint)
        .service(search_comment);
}

crate::post_funcs! {
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (view_info, "/view_info", ViewInfoRequest, ViewInfoResponse),
    (modify_password, "/modify_password", ModifyPasswordRequest, SimpleResponse),
    (modify_info, "/modify_info", ModifyInfoRequest, SimpleResponse),
    (add_time, "/add_time", AddTimeRequest, SimpleResponse),
    (modify_time, "/modify_time", ModifyTimeRequest, SimpleResponse),
    (delete_time, "/delete_time", DeleteTimeRequest, SimpleResponse),
    (search_time, "/search_time", SearchTimeRequest, SearchTimeResponse),
    (search_appoint, "/search_appoint", SearchAppointRequest, SearchAppointResponse),
    (finish_appoint, "/finish_appoint", FinishAppointRequest, SimpleResponse),
    (search_comment, "/search_comment", SearchCommentRequest, SearchCommentResponse),
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
                .context("数据库错误")?;
            if res != 1 {
                bail!("密码错误");
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
    use crate::schema::doctor_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(doctor_logins::table.filter(doctor_logins::token.eq(info.login_token)))
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
    use crate::schema::doctors;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let res = doctors::table
        .filter(doctors::did.eq(&did))
        .get_result::<DoctorData>(&conn)
        .context("数据库错误")?;

    let data = ViewInfoResponse {
        success: true,
        err: "".to_string(),
        did: res.did,
        name: res.name,
        birthday: format!(
            "{}",
            res.birthday.unwrap_or(NaiveDate::from_ymd(1970, 1, 1))
        ),
        gender: res.gender,
        info: res.information,
    };
    Ok(data)
}

async fn modify_password_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyPasswordRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token.clone(), &pool).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let hashed_password_old =
                format!("{:x}", Blake2b::digest(info.password_old.as_bytes()));
            let res = doctors::table
                .filter(doctors::did.eq(&did))
                .filter(doctors::password.eq(&hashed_password_old))
                .count()
                .get_result::<i64>(&conn)
                .context("数据库错误")?;
            if res != 1 {
                bail!("密码错误");
            }

            let hashed_password_new =
                format!("{:x}", Blake2b::digest(info.password_new.as_bytes()));
            diesel::update(doctors::table.filter(doctors::did.eq(&did)))
                .set(doctors::password.eq(&hashed_password_new))
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
    use crate::schema::doctors;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token, &pool).await?;

    let mut data = UpdateDoctor {
        name: info.name,
        gender: info.gender,
        information: info.info,
        ..Default::default()
    };
    if let Some(birthday) = info.birthday {
        let birthday = NaiveDate::parse_from_str(&birthday, "%Y-%m-%d").context("生日格式错误")?;
        data.birthday = Some(birthday);
    }

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::update(doctors::table.filter(doctors::did.eq(did)))
            .set(&data)
            .execute(&conn)
    })
    .await
    .context("数据库错误")?;

    Ok(SimpleResponse::ok())
}

async fn add_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<AddTimeRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::times;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token.clone(), &pool).await?;

    let (start_time, end_time) = crate::utils::get_time_from_str(&info.date, &info.time)?;
    // if start_time >= end_time {
    //     bail!("非法的时间区间");
    // }

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
                .context("数据库错误")?;
            if res > 0 {
                bail!("时间区间与已有的冲突");
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
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn modify_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyTimeRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::times;

    let info = info.into_inner();
    get_did_from_token(info.login_token.clone(), &pool).await?;
    assert::assert_time(&pool, info.tid).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let time_data = times::table
                .filter(times::tid.eq(info.tid))
                .get_result::<TimeData>(&conn)
                .context("数据库错误")?;

            // if time_data.appointed > 0 && (info.start_time.is_some() || info.end_time.is_some()) {
            //     bail!("不能在已有用户预约的情况下修改时间")
            // }

            let mut data = UpdateTime::default();
            if let Some(cap) = info.capacity {
                if cap < time_data.appointed {
                    bail!(format!(
                        "不能修改容量为 {}, 因为已经有 {} 个用户预约了",
                        cap, time_data.appointed
                    ));
                } else {
                    data.capacity = Some(cap);
                }
            }

            // if let Some(start_time) = info.start_time {
            //     let start_time =
            //         crate::utils::parse_time_str(&start_time).context("起始时间格式错误")?;
            //     data.start_time = Some(start_time);
            // }
            // if let Some(end_time) = info.end_time {
            //     let end_time =
            //         crate::utils::parse_time_str(&end_time).context("结束时间格式错误")?;
            //     data.end_time = Some(end_time);
            // }

            diesel::update(times::table.filter(times::tid.eq(info.tid)))
                .set(&data)
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn delete_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<DeleteTimeRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::times;

    let info = info.into_inner();
    get_did_from_token(info.login_token, &pool).await?;
    let tid = info.tid;
    assert::assert_time(&pool, tid).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let time_data = times::table
                .filter(times::tid.eq(tid))
                .get_result::<TimeData>(&conn)
                .context("数据库错误")?;
            if time_data.appointed > 0 {
                bail!("不能在已有用户预约的情况下删除时间");
            }

            diesel::delete(times::table.filter(times::tid.eq(tid)))
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn search_time_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchTimeRequest>,
) -> anyhow::Result<SearchTimeResponse> {
    use crate::schema::times;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token, &pool).await?;

    let (start_time, end_time) = crate::utils::get_time_pair_from_date_opt(info.date)?;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(30).max(0);
    let tms = web::block(move || {
        times::table
            .filter(times::did.eq(&did))
            .filter(times::start_time.ge(&start_time))
            .filter(times::end_time.le(&end_time))
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
            date: data.start_time.date().format("%Y-%m-%d").to_string(),
            time: crate::utils::get_time_str(&data.start_time, &data.end_time).to_owned(),
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
    use crate::schema::{appointments, times, users};

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token, &pool).await?;

    let (start_time, end_time) =
        crate::utils::parse_time_pair_str_opt(info.start_time, info.end_time)?;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(30).max(0);
    let status = info.status;
    let appos = web::block(move || {
        times::table
            .filter(times::did.eq(&did))
            .filter(times::start_time.ge(&start_time))
            .filter(times::end_time.le(&end_time))
            .inner_join(appointments::table.on(times::tid.eq(appointments::tid)))
            .filter((appointments::status.eq(&status)).or(&status == "所有"))
            .inner_join(users::table.on(appointments::username.eq(users::username)))
            .order(times::start_time.desc())
            .offset(first_index)
            .limit(limit)
            .get_results::<(TimeData, Appointment, UserData)>(&conn)
    })
    .await
    .context("数据库错误")?;

    let appos = appos
        .into_iter()
        .map(|(time_data, appo_data, user_data)| SearchAppointItem {
            username: user_data.username,
            name: user_data.name,
            age: user_data
                .birthday
                .map_or(-1, |birth| Utc::now().year() - birth.year()),
            tid: time_data.tid,
            date: time_data.start_time.date().format("%Y-%m-%d").to_string(),
            time: crate::utils::get_time_str(&time_data.start_time, &time_data.end_time).to_owned(),
            status: appo_data.status,
            appo_time: crate::utils::format_time_str(&appo_data.time),
        })
        .collect();

    Ok(SearchAppointResponse {
        success: true,
        err: "".to_string(),
        appointments: appos,
    })
}

async fn finish_appoint_impl(
    pool: web::Data<DbPool>,
    info: web::Json<FinishAppointRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::{appointments, times};

    let info = info.into_inner();
    get_did_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let username = info.username;
    let tid = info.tid;
    web::block(move || {
        conn.transaction(|| {
            let appo_data = appointments::table
                .filter(appointments::username.eq(&username))
                .filter(appointments::tid.eq(&tid))
                .get_results::<Appointment>(&conn)
                .context("数据库错误")?;
            if appo_data.len() != 1 {
                bail!("科室不存在");
            }
            if appo_data[0].status != APPOINT_STATUS_UNFINISHED {
                bail!("只能完成未完成的预约");
            }

            let time_data = times::table
                .filter(times::tid.eq(&appo_data[0].tid))
                .get_result::<TimeData>(&conn)
                .context("数据库错误")?;
            let now = Utc::now().naive_utc();
            if now
                .signed_duration_since(time_data.start_time)
                .num_seconds()
                < 0
            {
                bail!("不能完成一个还在未来的预约");
            }

            diesel::update(
                appointments::table
                    .filter(appointments::username.eq(&username))
                    .filter(appointments::tid.eq(&tid)),
            )
            .set(appointments::status.eq(APPOINT_STATUS_FINISHED))
            .execute(&conn)
            .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn search_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchCommentRequest>,
) -> anyhow::Result<SearchCommentResponse> {
    use crate::schema::comments;

    let info = info.into_inner();
    let did = get_did_from_token(info.login_token, &pool).await?;

    let (start_time, end_time) =
        crate::utils::parse_time_pair_str_opt(info.start_time, info.end_time)?;

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(30).max(0);
    let cmts = web::block(move || {
        comments::table
            .filter(comments::did.eq(&did))
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
            time: crate::utils::format_time_str(&data.time),
        })
        .collect();

    Ok(SearchCommentResponse {
        success: true,
        err: "".to_string(),
        comments: cmts,
    })
}
