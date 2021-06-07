use crate::schema::users;
use chrono::NaiveDate;

#[derive(Queryable, Insertable, Identifiable)]
#[primary_key(username)]
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

#[derive(AsChangeset, Default)]
#[table_name = "users"]
pub struct UpdateUser {
    pub name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub telephone: Option<String>,
}
