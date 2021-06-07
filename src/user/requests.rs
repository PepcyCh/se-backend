use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub name: String,
    pub password: String,
    pub gender: String,
    #[serde(default)]
    pub birthday: String,
    #[serde(default)]
    pub telephone: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub login_token: String,
}

#[derive(Deserialize)]
pub struct ModifyPasswordRequest {
    pub login_token: String,
    pub password_old: String,
    pub password_new: String,
}

#[derive(Deserialize)]
pub struct ModifyInfoRequest {
    pub login_token: String,
    pub name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<String>,
    pub telephone: Option<String>,
}

#[derive(Deserialize)]
pub struct AppointRequest {
    pub login_token: String,
    pub tid: u64,
}

#[derive(Deserialize)]
pub struct CancelAppointRequest {
    pub login_token: String,
    pub tid: u64,
}

#[derive(Deserialize)]
pub struct CommentRequest {
    pub login_token: String,
    pub did: String,
    pub comment: String,
}

#[derive(Deserialize)]
pub struct DeleteCommentRequest {
    pub login_token: String,
    pub cid: u64,
}

#[derive(Deserialize)]
pub struct SearchDepartRequest {
    pub login_token: String,
    pub depart_name: String,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchDoctorRequest {
    pub login_token: String,
    pub depart_name: Option<String>,
    pub doctor_name: Option<String>,
    pub rank: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchCommentRequest {
    pub login_token: String,
    pub did: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchTimeRequest {
    pub login_token: String,
    pub did: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default)]
    pub show_all: bool,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchAppointRequest {
    pub login_token: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default = "search_appoint_request_status_default")]
    pub status: String,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

fn search_appoint_request_status_default() -> String {
    crate::models::appointments::APPOINT_STATUS_UNFINISHED.to_string()
}
