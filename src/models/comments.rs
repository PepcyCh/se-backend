use crate::schema::comments;
use chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Comment {
    pub cid: u64,
    pub username: String,
    pub did: String,
    pub comment: String,
    pub time: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "comments"]
pub struct NewComment {
    pub username: String,
    pub did: String,
    pub comment: String,
}
