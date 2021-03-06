use actix_web::web;
use anyhow::{bail, Context};
use diesel::prelude::*;

use crate::{database::get_db_conn, models::users::UserData, DbPool};

pub async fn assert_user(
    pool: &web::Data<DbPool>,
    username: String,
    check_ban: bool,
) -> anyhow::Result<()> {
    use crate::schema::users;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .get_results::<UserData>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res.len() != 1 {
        bail!("用户不存在");
    }

    if check_ban && res[0].is_banned {
        bail!("用户被封禁");
    }

    Ok(())
}

pub async fn assert_doctor(pool: &web::Data<DbPool>, did: String) -> anyhow::Result<()> {
    use crate::schema::doctors;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        doctors::table
            .filter(doctors::did.eq(did))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("医生不存在");
    }

    Ok(())
}

pub async fn assert_admin(pool: &web::Data<DbPool>, aid: String) -> anyhow::Result<()> {
    use crate::schema::administrators;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        administrators::table
            .filter(administrators::aid.eq(aid))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("管理员不存在");
    }

    Ok(())
}

pub async fn assert_depart(pool: &web::Data<DbPool>, depart_name: String) -> anyhow::Result<()> {
    use crate::schema::departments;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        departments::table
            .filter(departments::depart_name.eq(depart_name))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("科室不存在");
    }

    Ok(())
}

pub async fn assert_comment(pool: &web::Data<DbPool>, cid: u64) -> anyhow::Result<()> {
    use crate::schema::comments;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        comments::table
            .filter(comments::cid.eq(cid))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("评论不存在");
    }

    Ok(())
}

pub async fn assert_time(pool: &web::Data<DbPool>, tid: u64) -> anyhow::Result<()> {
    use crate::schema::times;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        times::table
            .filter(times::tid.eq(tid))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("时间段不存在");
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn assert_appoint(
    pool: &web::Data<DbPool>,
    username: String,
    tid: u64,
) -> anyhow::Result<()> {
    use crate::schema::appointments;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        appointments::table
            .filter(appointments::username.eq(username))
            .filter(appointments::tid.eq(tid))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("数据库错误")?;

    if res == 0 {
        bail!("预约不存在");
    }

    Ok(())
}
