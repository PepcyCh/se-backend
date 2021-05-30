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

#[get("/")]
async fn user_index() -> impl Responder {
    NamedFile::open("user/index.html")
}

#[get("/{filename}")]
async fn user_default_get(web::Path(filename): web::Path<String>) -> impl Responder {
    NamedFile::open(format!("user/{}", filename))
}

#[get("/")]
async fn doctor_index() -> impl Responder {
    NamedFile::open("doctor/index.html")
}

#[get("/{filename}")]
async fn doctor_default_get(web::Path(filename): web::Path<String>) -> impl Responder {
    NamedFile::open(format!("doctor/{}", filename))
}

#[get("/")]
async fn admin_index() -> impl Responder {
    NamedFile::open("admin/index.html")
}

#[get("/{filename}")]
async fn admin_default_get(web::Path(filename): web::Path<String>) -> impl Responder {
    NamedFile::open(format!("admin/{}", filename))
}

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
                    .service(user_index)
                    .service(user_default_get)
                    .configure(user::config),
            )
            // doctor
            .service(
                web::scope("/doctor")
                    .service(doctor_index)
                    .service(doctor_default_get)
                    .configure(doctor::config),
            )
            // administrator
            .service(
                web::scope("/admin")
                    .service(admin_index)
                    .service(admin_default_get)
                    .configure(admin::config),
            )
    })
    .bind(bind)?
    .run()
    .await
}
