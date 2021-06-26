#[macro_export]
macro_rules! post_funcs {
    ( $( ( $func_name:ident, $url:expr, $request:ty, $response:ty ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                #[post($url)]
                async fn $func_name(
                    pool: web::Data<DbPool>,
                    info: web::Json<$request>
                ) -> impl Responder {
                    let response = match [<$func_name _impl>](pool, info).await {
                        Ok(response) => response,
                        Err(err) => $response::err(err.to_string()),
                    };
                    HttpResponse::Ok().json(response)
                }
            }
        )+
    };
}

use anyhow::{bail, Context};
use chrono::{DateTime, NaiveDateTime, NaiveTime};

pub fn assert_gender_str(gender: &str) -> anyhow::Result<()> {
    if gender != "男" && gender != "女" {
        bail!("性别格式错误")
    }
    Ok(())
}

pub fn parse_time_str<S: AsRef<str>>(s: S) -> anyhow::Result<NaiveDateTime> {
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S%.f%:z";
    const TIME_FMT_SPECIAL: &str = "%Y-%m-%dT%H:%M:%S%.fZ";

    let s = s.as_ref();
    if let Some('Z') = s.chars().last() {
        NaiveDateTime::parse_from_str(s.as_ref(), TIME_FMT_SPECIAL).context("未知错误")
    } else {
        DateTime::parse_from_str(s.as_ref(), TIME_FMT)
            .context("未知错误")
            .map(|t| t.naive_utc())
    }
}

pub fn parse_time_pair_str_opt<S1: AsRef<str>, S2: AsRef<str>>(
    start_time: Option<S1>,
    end_time: Option<S2>,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    let time_min = parse_time_str("1901-01-01T00:00:00.0000Z")?;
    let time_max = parse_time_str("2901-01-01T00:00:00.0000Z")?;
    let start_time = start_time.map_or(Ok(time_min.clone()), |t| {
        crate::utils::parse_time_str(t).context("起始时间格式错误")
    })?;
    let end_time = end_time.map_or(Ok(time_max.clone()), |t| {
        crate::utils::parse_time_str(t).context("结束时间格式错误")
    })?;
    Ok((start_time, end_time))
}

pub fn parse_time_pair_str<S1: AsRef<str>, S2: AsRef<str>>(
    start_time: S1,
    end_time: S2,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    let start_time = parse_time_str(start_time).context("起始时间格式错误")?;
    let end_time = parse_time_str(end_time).context("结束时间格式错误")?;
    Ok((start_time, end_time))
}

pub fn get_time_pair_from_date_opt<S: AsRef<str>>(
    date: Option<S>,
) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    if let Some(date) = date {
        let start_time_str = format!("{}T00:00:00+00:00", date.as_ref());
        let end_time_str = format!("{}T23:59:59+00:00", date.as_ref());
        crate::utils::parse_time_pair_str(start_time_str, end_time_str).context("日期格式错误")
    } else {
        crate::utils::parse_time_pair_str_opt::<String, String>(None, None)
    }
}

pub fn format_time_str(time: &NaiveDateTime) -> String {
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S%.f";

    format!("{}+00:00", time.format(TIME_FMT))
}

pub fn get_str_pattern<S: AsRef<str>>(s: S) -> String {
    format!("%{}%", s.as_ref())
}

pub fn get_str_pattern_opt<S: AsRef<str>>(s: Option<S>) -> String {
    match s {
        Some(s) => get_str_pattern(s),
        None => "%".to_string(),
    }
}

pub fn get_time_from_str(date: &str, time: &str) -> anyhow::Result<(NaiveDateTime, NaiveDateTime)> {
    match time {
        crate::models::times::TIME_AM => {
            let start_time_str = format!("{}T09:00:00+00:00", date);
            let end_time_str = format!("{}T11:00:00+00:00", date);
            crate::utils::parse_time_pair_str(start_time_str, end_time_str).context("日期格式错误")
        }
        crate::models::times::TIME_PM => {
            let start_time_str = format!("{}T15:00:00+00:00", date);
            let end_time_str = format!("{}T17:00:00+00:00", date);
            crate::utils::parse_time_pair_str(start_time_str, end_time_str).context("日期格式错误")
        }
        _ => Err(anyhow::anyhow!("时间格式错误")),
    }
}

pub fn get_time_str(_start_time: &NaiveDateTime, end_time: &NaiveDateTime) -> &'static str {
    if end_time.time() >= NaiveTime::from_hms(12, 0, 0) {
        crate::models::times::TIME_PM
    } else {
        crate::models::times::TIME_AM
    }
}
