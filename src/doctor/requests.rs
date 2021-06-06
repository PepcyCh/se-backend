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
pub struct AddTimeRequest {
    pub login_token: String,
    pub start_time: String,
    pub end_time: String,
    pub capacity: i32,
}
