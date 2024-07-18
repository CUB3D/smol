use crate::endpoint_admin::admin;
use crate::endpoint_api_link_info::api_link_info;
use crate::endpoint_favicon::favicon;
use crate::endpoint_index::{index, index_head};
use crate::models::NewLink;
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::config::CookieContentSecurity;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{Key, SameSite};
use actix_web::http::header::LOCATION;
use actix_web::middleware::{Compress, Logger, NormalizePath, TrailingSlash};
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, HandleError, Pool};
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use rand::Rng;
use serde::Deserialize;
use std::env;
use std::fmt::Debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use url::{ParseError, Url};
use uuid::Uuid;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub mod endpoint_admin;
pub mod endpoint_api_link_info;
pub mod endpoint_favicon;
pub mod endpoint_index;
pub mod models;
pub mod schema;

#[derive(Deserialize, Debug)]
struct ShortenPayload {
    pub source: String,
}

type DBHandle = Pool<ConnectionManager<MysqlConnection>>;

#[post("/api/shorten")]
async fn api_shorten(pool: Data<DBHandle>, json: web::Json<ShortenPayload>) -> impl Responder {
    use schema::links;

    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Shorten API", request_id = %request_id);
    let _guard = span.enter();

    let mut source_url = json.source.clone();

    let source_parse = Url::parse(&source_url);
    tracing::info!("Parsed url {:?} as {:?}", source_url, source_parse);

    // If the given url is missing the scheme then try adding it
    if let Err(ParseError::RelativeUrlWithoutBase) = source_parse {
        tracing::info!("Url missing scheme, trying to parse as https");
        let http_url = format!("https://{}", &source_url);
        match Url::parse(&http_url) {
            Ok(http_url) => {
                source_url = http_url.to_string();
            }
            Err(e) => {
                tracing::info!("Failed to parse: {e:?}");
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    let short_code: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .map(char::from)
        .take(3)
        .collect();

    tracing::info!("Generated shortcode {}", short_code);

    let new_link = NewLink {
        short_code: &short_code,
        original_link: source_url.as_str(),
    };

    match pool.get() {
        Ok(mut conn) => {
            match diesel::insert_into(links::table)
                .values(&new_link)
                .execute(&mut conn)
            {
                Ok(_) => {
                    tracing::info!("Inserted ok");
                    HttpResponse::Ok().body(short_code)
                }
                Err(e) => {
                    tracing::error!("Failed to execute query {e:?}");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get pool {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/{shortId}")]
async fn short(pool: Data<DBHandle>, path: web::Path<(String,)>) -> impl Responder {
    use self::models::Link;
    use schema::links::dsl::*;

    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Get short URL", request_id = %request_id);
    let _guard = span.enter();

    tracing::info!("Given link: {}", &path.0);

    if let Ok(mut conn) = pool.get() {
        let other_short = links
            .filter(short_code.eq(&path.0))
            .first::<Link>(&mut conn);

        if let Ok(short) = other_short {
            HttpResponse::PermanentRedirect()
                .append_header((LOCATION, short.original_link))
                .finish()
        } else {
            HttpResponse::PermanentRedirect()
                .append_header((LOCATION, "/"))
                .finish()
        }
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

fn get_db_connection() -> Pool<ConnectionManager<MysqlConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut conn = MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Unable to run migrations");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    #[derive(Debug, Default)]
    struct ErrorHandler;

    impl<T: Debug> HandleError<T> for ErrorHandler {
        fn handle_error(&self, error: T) {
            tracing::error!("Got error {:?}", error);
            panic!("error");
        }
    }

    diesel::r2d2::Pool::builder()
        .error_handler(Box::new(ErrorHandler))
        .max_size(4)
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to build db connection pool")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0:8080".into());
    tracing::info!("Server launched on '{}'", host);

    let pkey = env::var("PRIVATE_KEY")
        .map(|s| s.into_bytes())
        .expect("No PRIVATE_KEY in env");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(get_db_connection()))
            .service(Files::new("/static", "./static"))
            .service(index)
            .service(index_head)
            .service(favicon)
            .service(admin)
            .service(api_shorten)
            .service(api_link_info)
            .service(short)
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(IdentityMiddleware::default())
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&pkey))
                    .cookie_name("smol-auth".to_string())
                    .cookie_secure(false)
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_same_site(SameSite::Lax)
                    .build(),
            )
    })
    .bind(host)
    .expect("Unable to bind to address")
    .run()
    .await
}
