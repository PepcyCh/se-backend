use serde::Serialize;

#[derive(Default, Serialize)]
pub struct SimpleResponse {
    pub success: bool,
    pub err: String,
}

impl SimpleResponse {
    pub fn ok() -> Self {
        Self {
            success: true,
            err: "".to_string(),
        }
    }
}

#[macro_export]
macro_rules! impl_err_response {
    ( $( $type:ty),+ $(,)? ) => {
        $(
            impl $type {
                pub fn err<S: ToString>(err: S) -> Self {
                    Self {
                        success: false,
                        err: err.to_string(),
                        ..Default::default()
                    }
                }
            }
        )+
    };
}

impl_err_response! {
    SimpleResponse,
}
