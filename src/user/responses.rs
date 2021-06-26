use serde::Serialize;

#[derive(Default, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub err: String,
    pub login_token: String,
}

#[derive(Default, Serialize)]
pub struct ViewInfoResponse {
    pub success: bool,
    pub err: String,
    pub username: String,
    pub name: String,
    pub gender: String,
    pub id_number: String,
    pub birthday: String,
    pub telephone: String,
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
    pub rank: String,
    pub gender: String,
    pub age: i32,
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
    pub date: String,
    pub time: String,
    pub did: String,
    pub doctor_name: String,
    pub doctor_depart: String,
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
    pub doctor_depart: String,
    pub tid: u64,
    pub date: String,
    pub time: String,
    pub status: String,
    pub appo_time: String,
}

#[derive(Default, Serialize)]
pub struct SearchAppointResponse {
    pub success: bool,
    pub err: String,
    pub appointments: Vec<SearchAppointItem>,
}

crate::impl_err_response! {
    LoginResponse,
    ViewInfoResponse,
    SearchDepartResponse,
    SearchDoctorResponse,
    SearchCommentResponse,
    SearchTimeResponse,
    SearchAppointResponse,
}
