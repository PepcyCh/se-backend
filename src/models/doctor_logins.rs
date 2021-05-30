use crate::schema::doctor_logins;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[table_name = "doctor_logins"]
pub struct DoctorLoginData {
    pub token: String,
    pub did: String,
    pub login_time: NaiveDateTime,
}
