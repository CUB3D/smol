use actix_web::{App, HttpResponse, HttpServer, Responder, get, head, post, web};
use serde::Deserialize;
use rand::Rng;

#[head("/")]
async fn index_head() -> impl Responder { HttpResponse::Ok() }

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/index.html"))
}


#[derive(Deserialize, Debug)]
struct ShortenPayload {
    pub source: String
}

#[post("/api/shorten")]
async fn api_shorten(
    json: web::Json<ShortenPayload>
) -> impl Responder {
    let short_code: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(3)
        .collect();

    HttpResponse::Ok().body(short_code)
}

#[get("/{short}")]
async fn short(
    path: web::Path<(String)>
) -> impl Responder {
    HttpResponse::PermanentRedirect()
        .header("Location", "https://google.com")
        .finish()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(index_head)
            .service(short)
            .service(api_shorten)
    })
    .bind("0.0.0.0:8080").expect("Unable to bind to address")
    .run()
    .await
}
