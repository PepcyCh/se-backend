#[macro_use]
extern crate diesel;

mod admin;
mod database;
mod doctor;
mod models;
mod protocol;
mod schema;
mod user;
mod utils;

use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Responder};
use diesel::{r2d2::ConnectionManager, MysqlConnection};

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let conn_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let manager = ConnectionManager::<MysqlConnection>::new(conn_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let bind = "127.0.0.1:8080";

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            // user
            .service(
                web::scope("/user")
                    .configure(user::config),
            )
            // doctor
            .service(
                web::scope("/doctor")
                    .configure(doctor::config),
            )
            // administrator
            .service(
                web::scope("/admin")
                    .configure(admin::config),
            )
    })
    .bind(bind)?
    .run()
    .await
}
