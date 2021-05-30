use crate::schema::users;
use chrono::NaiveDate;

#[derive(Queryable, Insertable)]
#[table_name = "users"]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub name: String,
    pub gender: String,
    pub birthday: Option<NaiveDate>,
    pub telephone: String,
    pub is_banned: bool,
}
