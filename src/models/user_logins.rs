use crate::schema::user_logins;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[table_name = "user_logins"]
pub struct UserLoginData {
    pub token: String,
    pub username: String,
    pub login_time: NaiveDateTime,
}
