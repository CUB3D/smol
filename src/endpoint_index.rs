use actix_web::{HttpResponse, Responder};
use actix_web::{get, head};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

#[head("/")]
pub async fn index_head() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body(IndexTemplate {}.render().expect("Failed to render index"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::Method;
    use actix_web::{App, test};

    #[actix_rt::test]
    async fn test_index_ok() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_head_ok() {
        let app = test::init_service(App::new().service(index_head)).await;
        let req = test::TestRequest::default()
            .method(Method::HEAD)
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
