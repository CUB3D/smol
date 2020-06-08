use crate::models::NewLink;
use actix_files::Files;
use actix_web::http::header::LOCATION;
use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::web::Data;
use actix_web::{get, head, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use dotenv::dotenv;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use url::{ParseError, Url};

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;
embed_migrations!();

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

#[get("/favicon.ico")]
async fn favicon() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize, Debug)]
struct ShortenPayload {
    pub source: String,
}

type DBHandle = Pool<ConnectionManager<MysqlConnection>>;

#[post("/api/shorten")]
async fn api_shorten(pool: Data<DBHandle>, json: web::Json<ShortenPayload>) -> impl Responder {
    use schema::links;

    let short_code: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(3)
        .collect();

    let mut source_url = json.source.clone();

    let source_parse = Url::parse(&source_url);

    // If the given url is missing the scheme then try adding it
    if let Err(ParseError::RelativeUrlWithoutBase) = source_parse {
        let http_url = format!("https://{}", &source_url);
        if let Ok(http_url) = Url::parse(&http_url) {
            source_url = http_url.to_string();
        } else {
            return HttpResponse::BadRequest().finish();
        }
    }

    let new_link = NewLink {
        short_code: &short_code,
        original_link: source_url.as_str(),
    };

    let conn = pool.get().unwrap();

    diesel::insert_into(links::table)
        .values(&new_link)
        .execute(&conn)
        .expect("Unable to add link");

    HttpResponse::Ok().body(short_code)
}

#[get("/{shortId}")]
async fn short(pool: Data<DBHandle>, path: web::Path<(String,)>) -> impl Responder {
    use self::models::Link;
    use schema::links::dsl::*;

    println!("Given link: {}", &path.0);

    let conn = pool.get().unwrap();

    let other_short = links.filter(short_code.eq(&path.0)).first::<Link>(&conn);

    if let Ok(short) = other_short {
        HttpResponse::PermanentRedirect()
            .header(LOCATION, short.original_link)
            .finish()
    } else {
        HttpResponse::PermanentRedirect()
            .header(LOCATION, "/")
            .finish()
    }
}

#[derive(Serialize)]
struct LinkInfo {
    target: String,
    created: String,
}

#[get("/api/link/{shortId}/info")]
async fn api_link_info(pool: Data<DBHandle>, path: web::Path<(String,)>) -> impl Responder {
    use self::models::Link;
    use schema::links::dsl::*;

    let conn = pool.get().unwrap();

    let other_short = links.filter(short_code.eq(&path.0)).first::<Link>(&conn);

    if let Ok(s) = other_short {
        let link_info = LinkInfo {
            target: s.original_link,
            created: s.created.to_string(),
        };

        HttpResponse::Ok().json(link_info)
    } else {
        HttpResponse::BadRequest().finish()
    }
}

fn get_db_connection() -> Pool<ConnectionManager<MysqlConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .expect("Unable to run migrations");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(4)
        .test_on_check_out(true)
        .build(manager)
        .unwrap();

    pool
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(get_db_connection())
            .service(index)
            .service(index_head)
            .service(favicon)
            .service(short)
            .service(api_shorten)
            .service(api_link_info)
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::default())
    })
    .bind(env::var("HOST").unwrap_or("0.0.0.0:8080".into()))
    .expect("Unable to bind to address")
    .run()
    .await
}
