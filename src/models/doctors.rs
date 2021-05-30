use crate::schema::doctors;
use chrono::NaiveDate;

#[derive(Queryable, Insertable)]
#[table_name = "doctors"]
pub struct DoctorData {
    pub did: String,
    pub name: String,
    pub password: String,
    pub gender: String,
    pub birthday: Option<NaiveDate>,
    pub department: String,
    #[column_name = "rankk"]
    pub rank: String,
    pub infomation: Option<String>,
}
