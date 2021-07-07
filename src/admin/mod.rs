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
        users::{UpdateUser, UserData},
    },
    protocol::SimpleResponse,
    DbPool,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{bail, Context};
use blake2::{Blake2b, Digest};
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;

use self::{requests::*, responses::*, utils::get_aid_from_token};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(add_doctor)
        .service(search_doctor)
        .service(view_doctor)
        .service(modify_doctor)
        .service(add_depart)
        .service(search_depart)
        .service(modify_depart)
        .service(search_comment)
        .service(delete_comment)
        .service(search_user)
        .service(view_user)
        .service(ban_user)
        .service(modify_user);
}

crate::post_funcs! {
    (register, "/register", RegisterRequest, SimpleResponse),
    (login, "/login", LoginRequest, LoginResponse),
    (logout, "/logout", LogoutRequest, SimpleResponse),
    (modify_password, "/modify_password", ModifyPasswordRequest, SimpleResponse),
    (add_doctor, "/add_doctor", AddDoctorRequest, SimpleResponse),
    (search_doctor, "/search_doctor", SearchDoctorRequest, SearchDoctorResponse),
    (view_doctor, "/view_doctor", ViewDoctorRequest, ViewDoctorResponse),
    (modify_doctor, "/modify_doctor", ModifyDoctorRequest, SimpleResponse),
    (add_depart, "/add_depart", AddDepartRequst, SimpleResponse),
    (search_depart, "/search_depart", SearchDepartRequest, SearchDepartResponse),
    (modify_depart, "/modify_depart", ModifyDepartRequest, SimpleResponse),
    (search_comment, "/search_comment", SearchCommentRequest, SearchCommentResponse),
    (delete_comment, "/delete_comment", DeleteCommentRequest, SimpleResponse),
    (search_user, "/search_user", SearchUserRequest, SearchUserResponse),
    (view_user, "/view_user", ViewUserRequest, ViewUserResponse),
    (ban_user, "/ban_user", BanUserRequest, SimpleResponse),
    (modify_user, "/modify_user", ModifyUserRequest, SimpleResponse),
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
                .context("数据库错误")?;
            if res > 0 {
                bail!("ID 重复");
            }

            let hashed_password = format!("{:x}", Blake2b::digest(info.password.as_bytes()));
            let data = AdminData {
                aid: info.aid,
                password: hashed_password,
            };
            diesel::insert_into(administrators::table)
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
                .context("数据库错误")?;

            if res != 1 {
                bail!("密码错误");
            }

            let login_token = crate::utils::generate_login_token(&info.aid, "admin");
            let token_data = AdminLoginData {
                token: login_token.clone(),
                aid: info.aid,
                login_time: Utc::now().naive_utc(),
            };
            diesel::insert_into(admin_logins::table)
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
    use crate::schema::admin_logins;

    let info = info.into_inner();
    let conn = get_db_conn(&pool)?;
    web::block(move || {
        diesel::delete(admin_logins::table.filter(admin_logins::token.eq(info.login_token)))
            .execute(&conn)
    })
    .await
    .context("数据库错误")?;

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
                .context("数据库错误")?;
            if res != 1 {
                bail!("密码错误");
            }

            let hashed_password_new =
                format!("{:x}", Blake2b::digest(info.password_new.as_bytes()));
            diesel::update(administrators::table.filter(administrators::aid.eq(&aid)))
                .set(administrators::password.eq(&hashed_password_new))
                .execute(&conn)
                .context("数据库错误")?;

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
                .context("数据库错误")?;
            if res > 0 {
                bail!("ID 重复");
            }

            crate::utils::assert_gender_str(&info.gender)?;

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
                .context("数据库错误")?;

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
    // get_aid_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let depart_name_pattern = crate::utils::get_str_pattern_opt(info.depart_name);
    let doctor_name_pattern = crate::utils::get_str_pattern_opt(info.doctor_name);
    let rank = crate::utils::get_str_pattern_opt(info.rank);
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

async fn view_doctor_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ViewDoctorRequest>,
) -> anyhow::Result<ViewDoctorResponse> {
    use crate::schema::doctors;

    let info = info.into_inner();
    assert::assert_doctor(&pool, info.did.clone()).await?;

    let conn = get_db_conn(&pool)?;
    let did = info.did;
    let data = web::block(move || {
        doctors::table
            .filter(doctors::did.eq(did))
            .get_result::<DoctorData>(&conn)
    })
    .await
    .context("数据库错误")?;

    Ok(ViewDoctorResponse {
        success: true,
        err: "".to_string(),
        did: data.did,
        name: data.name,
        birthday: format!(
            "{}",
            data.birthday.unwrap_or(NaiveDate::from_ymd(1970, 1, 1))
        ),
        gender: data.gender,
        depart: data.department,
        rank: data.rank,
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
        let birthday = NaiveDate::parse_from_str(&birthday, "%Y-%m-%d").context("生日格式错误")?;
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
    .context("数据库错误")?;

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
                .context("数据库错误")?;
            if res > 0 {
                bail!("科室名称重复");
            }

            let data = DepartData {
                depart_name,
                information,
            };
            diesel::insert_into(departments::table)
                .values(data)
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn search_depart_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchDepartRequest>,
) -> anyhow::Result<SearchDepartResponse> {
    use crate::schema::departments;

    let info = info.into_inner();
    // get_aid_from_token(info.login_token, &pool).await?;

    let conn = get_db_conn(&pool)?;
    let name_pattern = crate::utils::get_str_pattern_opt(info.depart_name);
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
        .context("数据库错误")?;
    }

    Ok(SimpleResponse::ok())
}

async fn search_comment_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchCommentRequest>,
) -> anyhow::Result<SearchCommentResponse> {
    use crate::schema::comments;

    let info = info.into_inner();
    // get_aid_from_token(info.login_token, &pool).await?;

    let (start_time, end_time) =
        crate::utils::parse_time_pair_str_opt(info.start_time, info.end_time)?;

    let conn = get_db_conn(&pool)?;
    let did = info.did;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
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
    .context("数据库错误")?;

    Ok(SimpleResponse::ok())
}

async fn search_user_impl(
    pool: web::Data<DbPool>,
    info: web::Json<SearchUserRequest>,
) -> anyhow::Result<SearchUserResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    // get_aid_from_token(info.login_token, &pool).await?;

    let username_pattern = crate::utils::get_str_pattern_opt(info.username);

    let conn = get_db_conn(&pool)?;
    let first_index = info.first_index.unwrap_or(0).max(0);
    let limit = info.limit.unwrap_or(10).max(0);
    let usrs = web::block(move || {
        users::table
            .filter(users::username.like(username_pattern))
            .order(users::username.asc())
            .offset(first_index)
            .limit(limit)
            .get_results::<UserData>(&conn)
    })
    .await
    .context("数据库错误")?;

    let usrs = usrs
        .into_iter()
        .map(|data| SearchUserItem {
            username: data.username,
            name: data.name,
            age: data
                .birthday
                .map_or(-1, |birth| Utc::now().year() - birth.year()),
            gender: data.gender,
            telephone: data.telephone,
            is_banned: data.is_banned,
        })
        .collect();

    Ok(SearchUserResponse {
        success: true,
        err: "".to_string(),
        users: usrs,
    })
}

async fn view_user_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ViewUserRequest>,
) -> anyhow::Result<ViewUserResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    // get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, info.username.clone(), false).await?;

    let conn = get_db_conn(&pool)?;
    let username = info.username;
    let data = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .get_result::<UserData>(&conn)
    })
    .await
    .context("数据库错误")?;

    Ok(ViewUserResponse {
        success: true,
        err: "".to_string(),
        username: data.username,
        name: data.name,
        birthday: format!(
            "{}",
            data.birthday.unwrap_or(NaiveDate::from_ymd(1970, 1, 1))
        ),
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
                .context("数据库错误")?;

            if data.is_banned && is_banned {
                bail!("用户已被封禁");
            }
            if !data.is_banned && !is_banned {
                bail!("用户已被解封");
            }

            diesel::update(users::table.filter(users::username.eq(&username)))
                .set(users::is_banned.eq(is_banned))
                .execute(&conn)
                .context("数据库错误")?;

            Ok(())
        })
    })
    .await?;

    Ok(SimpleResponse::ok())
}

async fn modify_user_impl(
    pool: web::Data<DbPool>,
    info: web::Json<ModifyUserRequest>,
) -> anyhow::Result<SimpleResponse> {
    use crate::schema::users;

    let info = info.into_inner();
    get_aid_from_token(info.login_token, &pool).await?;
    assert::assert_user(&pool, info.username.clone(), false).await?;

    let mut data = UpdateUser {
        name: info.name,
        gender: info.gender,
        id_number: info.id_number,
        telephone: info.telephone,
        ..Default::default()
    };
    if let Some(birthday) = info.birthday {
        let birthday = NaiveDate::parse_from_str(&birthday, "%Y-%m-%d").context("生日格式错误")?;
        data.birthday = Some(birthday);
    }

    let conn = get_db_conn(&pool)?;
    let username = info.username;
    web::block(move || {
        diesel::update(users::table.filter(users::username.eq(username)))
            .set(&data)
            .execute(&conn)
    })
    .await
    .context("数据库错误")?;

    Ok(SimpleResponse::ok())
}
