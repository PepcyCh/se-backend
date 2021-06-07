use serde::Serialize;

#[derive(Default, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub err: String,
    pub login_token: String,
}

#[derive(Default, Serialize)]
pub struct SearchAppointItem {
    pub username: String,
    pub name: String,
    pub age: i32,
    pub tid: u64,
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
    SearchAppointResponse,
    SearchCommentResponse,
}