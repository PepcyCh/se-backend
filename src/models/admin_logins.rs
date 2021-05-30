use crate::schema::admin_logins;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[table_name = "admin_logins"]
pub struct AdminLoginData {
    pub token: String,
    pub aid: String,
    pub login_time: NaiveDateTime,
}
