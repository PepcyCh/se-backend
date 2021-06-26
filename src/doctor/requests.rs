use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub did: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub login_token: String,
}

#[derive(Deserialize)]
pub struct ViewInfoRequest {
    pub login_token: String,
}

#[derive(Deserialize)]
pub struct AddTimeRequest {
    pub login_token: String,
    pub date: String,
    pub time: String,
    pub capacity: i32,
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
    pub birthday: Option<String>,
    pub gender: Option<String>,
    pub info: Option<String>,
}

#[derive(Deserialize)]
pub struct ModifyTimeRequest {
    pub login_token: String,
    pub tid: u64,
    pub capacity: Option<i32>,
}

#[derive(Deserialize)]
pub struct DeleteTimeRequest {
    pub login_token: String,
    pub tid: u64,
}

#[derive(Deserialize)]
pub struct SearchTimeRequest {
    pub login_token: String,
    pub date: Option<String>,
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

#[derive(Deserialize)]
pub struct FinishAppointRequest {
    pub login_token: String,
    pub username: String,
    pub tid: u64,
}

#[derive(Deserialize)]
pub struct SearchCommentRequest {
    pub login_token: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

fn search_appoint_request_status_default() -> String {
    crate::models::appointments::APPOINT_STATUS_UNFINISHED.to_string()
}
