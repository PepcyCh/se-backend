use crate::schema::appointments;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[table_name = "appointments"]
pub struct Appointment {
    pub username: String,
    pub tid: u64,
    pub status: String,
    pub time: Option<NaiveDateTime>,
}
