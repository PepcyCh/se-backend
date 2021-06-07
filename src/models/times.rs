use crate::schema::times;
use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct TimeData {
    pub tid: u64,
    pub did: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub capacity: i32,
    pub appointed: i32,
}

#[derive(Insertable)]
#[table_name = "times"]
pub struct NewTime {
    pub did: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub capacity: i32,
}

#[derive(AsChangeset, Default)]
#[table_name = "times"]
pub struct UpdateTime {
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub capacity: Option<i32>,
}
