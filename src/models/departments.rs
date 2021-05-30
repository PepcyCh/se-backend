use crate::schema::departments;

#[derive(Queryable, Insertable)]
#[table_name = "departments"]
pub struct DepartData {
    pub depart_name: String,
    pub infomation: String,
}
