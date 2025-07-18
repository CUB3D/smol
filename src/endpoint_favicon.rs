use actix_web::get;
use actix_web::{HttpResponse, Responder};

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_rt::test]
    async fn test_favicon_ok() {
        let app = test::init_service(App::new().service(favicon)).await;
        let req = test::TestRequest::get().uri("/favicon.ico").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
