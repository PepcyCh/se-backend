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

use anyhow::Context;
use chrono::{DateTime, NaiveDateTime};

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

pub fn format_time_str(time: &NaiveDateTime) -> String {
    const TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S%.f";

    format!("{}+00:00", time.format(TIME_FMT))
}
