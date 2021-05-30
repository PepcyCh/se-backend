use crate::schema::administrators;

#[derive(Queryable, Insertable)]
#[table_name = "administrators"]
pub struct AdminData {
    pub aid: String,
    pub password: String,
}
