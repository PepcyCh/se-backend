use actix_web::web;
use anyhow::{bail, Context};
use diesel::prelude::*;

use crate::{database::get_db_conn, DbPool};

pub async fn assert_user(pool: &web::Data<DbPool>, username: String) -> anyhow::Result<()> {
    use crate::schema::users;

    let conn = get_db_conn(pool)?;
    let res = web::block(move || {
        users::table
            .filter(users::username.eq(username))
            .count()
            .get_result::<i64>(&conn)
    })
    .await
    .context("DB error")?;

    if res == 0 {
        bail!("No such user");
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
    .context("DB error")?;

    if res == 0 {
        bail!("No such doctor");
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
    .context("DB error")?;

    if res == 0 {
        bail!("No such department");
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
    .context("DB error")?;

    if res == 0 {
        bail!("No such comment");
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
    .context("DB error")?;

    if res == 0 {
        bail!("No such time");
    }

    Ok(())
}

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
    .context("DB error")?;

    if res == 0 {
        bail!("No such appointment");
    }

    Ok(())
}
