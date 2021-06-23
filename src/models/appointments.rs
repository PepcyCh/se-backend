use crate::schema::appointments;
use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Appointment {
    pub username: String,
    pub tid: u64,
    pub status: String,
    pub time: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "appointments"]
pub struct NewAppointment {
    pub username: String,
    pub tid: u64,
    pub status: String,
    pub time: Option<NaiveDateTime>,
}

pub const APPOINT_STATUS_UNFINISHED: &str = "未完成";
pub const APPOINT_STATUS_FINISHED: &str = "已完成";
pub const APPOINT_STATUS_CANCELED: &str = "已取消";
