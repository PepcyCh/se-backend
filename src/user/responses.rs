use serde::Serialize;

#[derive(Default, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub err: String,
    pub login_token: String,
}

#[derive(Default, Serialize)]
pub struct SearchDepartItem {
    pub name: String,
    pub info: String,
}

#[derive(Default, Serialize)]
pub struct SearchDepartResponse {
    pub success: bool,
    pub err: String,
    pub departments: Vec<SearchDepartItem>,
}

#[derive(Default, Serialize)]
pub struct SearchDoctorItem {
    pub did: String,
    pub name: String,
    pub depart: String,
    pub gender: String,
    pub age: i64,
    pub info: String,
}

#[derive(Default, Serialize)]
pub struct SearchDoctorResponse {
    pub success: bool,
    pub err: String,
    pub doctors: Vec<SearchDoctorItem>,
}

#[derive(Default, Serialize)]
pub struct SearchCommentItem {
    pub cid: u64,
    pub username: String,
    pub comment: String,
    pub time: String,
}

#[derive(Default, Serialize)]
pub struct SearchCommentResponse {
    pub success: bool,
    pub err: String,
    pub comments: Vec<SearchCommentItem>,
}

#[derive(Default, Serialize)]
pub struct SearchTimeItem {
    pub tid: u64,
    pub start_time: String,
    pub end_time: String,
    pub capacity: i32,
    pub rest: i32,
}

#[derive(Default, Serialize)]
pub struct SearchTimeResponse {
    pub success: bool,
    pub err: String,
    pub times: Vec<SearchTimeItem>,
}

#[derive(Default, Serialize)]
pub struct SearchAppointItem {
    pub did: String,
    pub doctor_name: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub time: String,
}

#[derive(Default, Serialize)]
pub struct SearchAppointResponse {
    pub success: bool,
    pub err: String,
    pub appointments: Vec<SearchAppointItem>,
}

crate::impl_err_response! {
    LoginResponse,
    SearchDepartResponse,
    SearchDoctorResponse,
    SearchCommentResponse,
    SearchTimeResponse,
}
