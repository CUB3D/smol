use crate::models::NewLink;
use actix_files::Files;
use actix_web::middleware::{Compress, Logger};
use actix_web::{get, head, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use dotenv::dotenv;
use rand::Rng;
use serde::Deserialize;
use std::env;

#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

#[head("/")]
async fn index_head() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/index.html"))
}

#[derive(Deserialize, Debug)]
struct ShortenPayload {
    pub source: String,
}

#[post("/api/shorten")]
async fn api_shorten(
    conn: web::Data<MysqlConnection>,
    json: web::Json<ShortenPayload>,
) -> impl Responder {
    use schema::links;

    let short_code: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(3)
        .collect();

    let new_link = NewLink {
        short_code: &short_code,
        original_link: &json.source,
    };

    diesel::insert_into(links::table)
        .values(&new_link)
        .execute(conn.get_ref())
        // .get_result(conn.get_ref())
        .expect("Unable to add link");

    HttpResponse::Ok().body(short_code)
}

#[get("/{short}")]
async fn short(conn: web::Data<MysqlConnection>, path: web::Path<(String,)>) -> impl Responder {
    use self::models::Link;
    use schema::links::dsl::*;

    let short = links
        .filter(short_code.eq(&path.0))
        .first::<Link>(conn.get_ref())
        .expect("Unable to resolve code");

    HttpResponse::PermanentRedirect()
        .header("Location", short.original_link)
        .finish()
}

fn get_db_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(move || {
        App::new()
            .data(get_db_connection())
            .service(index)
            .service(index_head)
            .service(short)
            .service(api_shorten)
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
            .wrap(Compress::default())
    })
    .bind("0.0.0.0:8080")
    .expect("Unable to bind to address")
    .run()
    .await
}
