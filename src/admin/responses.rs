use serde::Serialize;

#[derive(Default, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub err: String,
    pub login_token: String,
}

crate::impl_err_response! {
    LoginResponse,
}
