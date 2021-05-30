use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub aid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub aid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub login_token: String,
}

#[derive(Deserialize)]
pub struct AddDoctorRequest {
    pub login_token: String,
    pub did: String,
    pub name: String,
    pub depart: String,
    pub rank: String,
    #[serde(default)]
    pub birthday: String,
    pub gender: String,
}
