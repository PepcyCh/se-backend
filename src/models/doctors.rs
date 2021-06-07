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
    pub information: String,
}

#[derive(AsChangeset, Default)]
#[table_name = "doctors"]
pub struct UpdateDoctor {
    pub name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub information: Option<String>,
    #[column_name = "rankk"]
    pub rank: Option<String>,
    pub department: Option<String>,
}
