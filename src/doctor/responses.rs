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
    pub did: String,
    pub name: String,
    pub birthday: String,
    pub gender: String,
    pub rankk: String,
    pub info: String,
    pub depart: String,
    pub depart_info: String,
}

#[derive(Default, Serialize)]
pub struct AddTimeResponse {
    pub success: bool,
    pub err: String,
    pub tid: u64,
}

#[derive(Default, Serialize)]
pub struct SearchTimeItem {
    pub tid: u64,
    pub date: String,
    pub time: String,
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
    pub username: String,
    pub name: String,
    pub age: i32,
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

crate::impl_err_response! {
    LoginResponse,
    ViewInfoResponse,
    AddTimeResponse,
    SearchTimeResponse,
    SearchAppointResponse,
    SearchCommentResponse,
}
