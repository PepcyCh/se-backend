mod requests;
mod responses;
mod utils;

use crate::{
    database::{assert, get_db_conn},
    models::{
        admin_logins::AdminLoginData,
        administrators::AdminData,
        comments::Comment,
        departments::DepartData,
        doctors::{DoctorData, UpdateDoctor},
        users::UserData,
    },
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;

use self::{requests::*, responses::*, utils::get_aid_from_token};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(add_doctor)
        .service(search_doctor)
        .service(modify_doctor)
        .service(add_depart)
        .service(modify_depart)
        .service(search_comment)
        .service(delete_comment)
        .service(view_user)
        .service(ban_user);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (modify_password, "/modify_password", ModifyPasswordRequest, SimpleResponse),
    (add_doctor, "/add_doctor", AddDoctorRequest, SimpleResponse),
    (search_doctor, "/search_doctor", SearchDoctorRequest, SearchDoctorResponse),
    (modify_doctor, "/modify_doctor", ModifyDoctorRequest, SimpleResponse),
    (add_depart, "/add_depart", AddDepartRequst, SimpleResponse),
    (modify_depart, "/modify_depart", ModifyDepartRequest, SimpleResponse),
    (search_comment, "/search_comment", SearchCommentRequest, SearchCommentResponse),
    (delete_comment, "/delete_comment", DeleteCommentRequest, SimpleResponse),
    (view_user, "/view_user", ViewUserRequest, ViewUserResponse),
    (ban_user, "/ban_user", BanUserRequest, SimpleResponse),
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

async fn modify_password_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyPasswordRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::administrators;

    let info = info.into_inner();
    let aid = get_aid_from_token(info.login_token.clone(), &pool).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        conn.transaction(|| {
            let hashed_password_old =
                format!("{:x}", Blake2b::digest(info.password_old.as_bytes()));
            let res = administrators::table
                .filter(administrators::aid.eq(&aid))
                .filter(administrators::password.eq(&hashed_password_old))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res != 1 {
                bail!("Wrong password");
            }

            let hashed_password_new =
                format!("{:x}", Blake2b::digest(info.password_new.as_bytes()));
            diesel::update(administrators::table.filter(administrators::aid.eq(&aid)))
                .set(administrators::password.eq(&hashed_password_new))
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

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
                information: "".to_string(),
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

async fn search_doctor_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchDoctorRequest>,
) -> anyhow::Result<SearchDoctorResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let depart_name_pattern = info
        .depart_name
        .map_or("%".to_string(), |s| format!("%{}%", s));
    let doctor_name_pattern = info
        .doctor_name
        .map_or("%".to_string(), |s| format!("%{}%", s));
    let rank = info.rank.or(Some("%".to_string())).unwrap();
    let first_index = info.first_index.or(Some(0)).unwrap().max(0);
    let limit = info.limit.or(Some(10)).unwrap().max(0);
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
    .context("DB error")?;

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

async fn modify_doctor_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyDoctorRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_doctor(&pool, info.did.clone()).await?;

    let mut data = UpdateDoctor {
        name: info.name,
        gender: info.gender,
        rank: info.rank,
        department: info.depart,
        ..Default::default()
    };
    if let Some(birthday) = info.birthday {
        let birthday = NaiveDate::parse_from_str(&birthday, "%Y-%m-%d")
            .context("Wrong format on 'birthday'")?;
        data.birthday = Some(birthday);
    }

    let conn = get_db_conn(&pool)?;
    let did = info.did;
    web::block(move || {
        diesel::update(doctors::table.filter(doctors::did.eq(did)))
            .set(&data)
            .execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn add_depart_impl(
    pool: web::Data<DbPool>,
    info: web::Json<AddDepartRequst>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::departments;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let depart_name = info.depart;
    let information = info.info;
    web::block(move || {
        conn.transaction(|| {
            let res = departments::table
                .filter(departments::depart_name.eq(&depart_name))
                .count()
                .get_result::<i64>(&conn)
                .context("DB error")?;
            if res > 0 {
                bail!("Duplicated department name");
            }

            let data = DepartData {
                depart_name,
                information,
            };
            diesel::insert_into(departments::table)
                .values(data)
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn modify_depart_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyDepartRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::departments;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_depart(&pool, info.depart.clone()).await?;

    if let Some(information) = info.info {
        let conn = get_db_conn(&pool)?;
        let depart = info.depart;
        web::block(move || {
            diesel::update(departments::table.filter(departments::depart_name.eq(depart)))
                .set(departments::information.eq(information))
                .execute(&conn)
        })
        .await
        .context("DB error")?;
    }

    Ok(SimpleResponse::ok())
}

async fn search_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchCommentRequest>,
) -> anyhow::Result<SearchCommentResponse> {
    use crate::schema::comments;
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S";

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;

    let time_min =
        NaiveDateTime::parse_from_str("1901-1-1T00:00:00", TIME_FMT).context("Unknown error")?;
    let time_max =
        NaiveDateTime::parse_from_str("2901-1-1T00:00:00", TIME_FMT).context("Unknown error")?;
    let start_time = info.start_time.map_or(Ok(time_min.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("Wrong format on 'start_time'")
    })?;
    let end_time = info.end_time.map_or(Ok(time_max.clone()), |t| {
        NaiveDateTime::parse_from_str(t.as_str(), TIME_FMT).context("Wrong format on 'end_time'")
    })?;

    let conn = get_db_conn(&pool)?;
    let did = info.did;
    let first_index = info.first_index.or(Some(0)).unwrap().max(0);
    let limit = info.limit.or(Some(10)).unwrap().max(0);
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
    .context("DB error")?;

    let cmts = cmts
        .into_iter()
        .map(|data| SearchCommentItem {
            cid: data.cid,
            username: data.username,
            comment: data.comment,
            time: format!("{}", data.time.format(TIME_FMT)),
        })
        .collect();

    Ok(SearchCommentResponse {
        success: true,
        err: "".to_string(),
        commenst: cmts,
    })
}

async fn delete_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<DeleteCommentRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::comments;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    let cid = info.cid;
    assert::assert_comment(&pool, cid).await?;

    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(comments::table.filter(comments::cid.eq(cid))).execute(&conn)
    })
    .await
    .context("DB error")?;

    Ok(SimpleResponse::ok())
}

async fn view_user_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ViewUserRequest>,
) -> anyhow::Result<ViewUserResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, info.username.clone(), false).await?;

    let conn = get_db_conn(&pool)?;
    let username = info.username;
    let data = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .get_result::<UserData>(&conn)
    })
    .await
    .context("DB error")?;

    Ok(ViewUserResponse {
        success: true,
        err: "".to_string(),
        username: data.username,
        name: data.name,
        age: data
            .birthday
            .map_or(-1, |birth| Utc::now().year() - birth.year()),
        gender: data.gender,
        telephone: data.telephone,
        is_banned: data.is_banned,
    })
}

async fn ban_user_impl(
    pool: web::Data<DbPool>,
    info: web::Json<BanUserRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, info.username.clone(), false).await?;

    let conn = get_db_conn(&pool)?;
    let username = info.username;
    let is_banned = info.is_banned;
    web::block(move || {
        conn.transaction(|| {
            let data = users::table
                .filter(users::username.eq(&username))
                .get_result::<UserData>(&conn)
                .context("DB error")?;

            if data.is_banned && is_banned {
                bail!("User has already been banned");
            }
            if !data.is_banned && !is_banned {
                bail!("User has already been un-banned");
            }

            diesel::update(users::table.filter(users::username.eq(&username)))
                .set(users::is_banned.eq(is_banned))
                .execute(&conn)
                .context("DB error")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}
