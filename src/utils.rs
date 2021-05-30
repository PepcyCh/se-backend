#[macro_export]
macro_rules! post_funcs {
    ( $( ( $func_name:ident, $url:expr, $request:ty, $response:ty ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                #[post($url)]
                async fn $func_name(
                    pool: web::Data<DbPool>,
                    info: web::Json<$request>
                ) -> impl Responder {
                    let response = match [<$func_name _impl>](pool, info).await {
                        Ok(response) => response,
                        Err(err) => $response::err(err.to_string()),
                    };
                    HttpResponse::Ok().json(response)
                }
            }
        )+
    };
}
