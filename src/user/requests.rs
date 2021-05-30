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
pub struct AppointRequest {
    pub login_token: String,
    pub tid: u64,
}