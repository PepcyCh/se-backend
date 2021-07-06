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
pub struct ModifyPasswordRequest {
    pub login_token: String,
    pub password_old: String,
    pub password_new: String,
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

#[derive(Deserialize)]
pub struct SearchDoctorRequest {
    // pub login_token: String,
    pub doctor_name: Option<String>,
    pub depart_name: Option<String>,
    pub rank: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ModifyDoctorRequest {
    pub login_token: String,
    pub did: String,
    pub name: Option<String>,
    pub gender: Option<String>,
    pub rank: Option<String>,
    pub depart: Option<String>,
    pub birthday: Option<String>,
}

#[derive(Deserialize)]
pub struct AddDepartRequst {
    pub login_token: String,
    pub depart: String,
    pub info: String,
}

#[derive(Deserialize)]
pub struct SearchDepartRequest {
    // pub login_token: String,
    pub depart_name: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ModifyDepartRequest {
    pub login_token: String,
    pub depart: String,
    pub info: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchCommentRequest {
    // pub login_token: String,
    pub did: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct DeleteCommentRequest {
    pub login_token: String,
    pub cid: u64,
}

#[derive(Deserialize)]
pub struct SearchUserRequest {
    pub username: Option<String>,
    pub first_index: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ViewUserRequest {
    // pub login_token: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct BanUserRequest {
    pub login_token: String,
    pub username: String,
    pub is_banned: bool,
}

#[derive(Deserialize)]
pub struct ModifyUserRequest {
    pub login_token: String,
    pub username: String,
    pub name: Option<String>,
    pub gender: Option<String>,
    pub id_number: Option<String>,
    pub birthday: Option<String>,
    pub telephone: Option<String>,
}
