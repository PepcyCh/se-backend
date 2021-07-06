use serde::Serialize;

#[derive(Default, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub err: String,
    pub login_token: String,
}

#[derive(Default, Serialize)]
pub struct SearchDoctorItem {
    pub did: String,
    pub name: String,
    pub gender: String,
    pub age: i32,
    pub depart: String,
    pub rank: String,
    pub info: String,
}

#[derive(Default, Serialize)]
pub struct SearchDoctorResponse {
    pub success: bool,
    pub err: String,
    pub doctors: Vec<SearchDoctorItem>,
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
    pub commenst: Vec<SearchCommentItem>,
}

#[derive(Default, Serialize)]
pub struct SearchUserItem {
    pub username: String,
    pub name: String,
    pub age: i32,
    pub gender: String,
    pub telephone: String,
    pub is_banned: bool,
}

#[derive(Default, Serialize)]
pub struct SearchUserResponse {
    pub success: bool,
    pub err: String,
    pub users: Vec<SearchUserItem>,
}

#[derive(Default, Serialize)]
pub struct ViewUserResponse {
    pub success: bool,
    pub err: String,
    pub username: String,
    pub name: String,
    pub birthday: String,
    pub gender: String,
    pub telephone: String,
    pub is_banned: bool,
}

crate::impl_err_response! {
    LoginResponse,
    SearchDoctorResponse,
    SearchDepartResponse,
    SearchCommentResponse,
    SearchUserResponse,
    ViewUserResponse,
}
