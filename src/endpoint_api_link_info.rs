use crate::models::Link;
use crate::schema::links::dsl::*;
use crate::DBHandle;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LinkInfo {
    pub target: String,
    pub created: String,
}

#[get("/api/link/{shortId}/info")]
pub async fn api_link_info(pool: Data<DBHandle>, path: web::Path<(String,)>) -> impl Responder {
    if let Ok(conn) = pool.get() {
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
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_db_connection;
    use actix_web::{test, App};
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_api_link_info() {
        dotenv().ok();

        let mut app =
            test::init_service(App::new().data(get_db_connection()).service(api_link_info)).await;
        let req = test::TestRequest::get()
            .uri("/api/link/gD9/info")
            .to_request();
        let resp: LinkInfo = test::read_response_json(&mut app, req).await;

        assert_eq!(resp.target, "https://example.com/");
    }
}
