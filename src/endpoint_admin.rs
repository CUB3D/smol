use crate::models::Link;
use crate::schema::links::dsl::*;
use crate::DBHandle;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use diesel::RunQueryDsl;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    pub links: Vec<Link>,
}

#[get("/admin")]
pub async fn admin(pool: Data<DBHandle>) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("Admin dashboard", request_id = %request_id);
    let _guard = span.enter();

    if let Ok(conn) = pool.get() {
        let db_links = links
            .load::<Link>(&conn)
            .expect("Failed to load Links from db");
        HttpResponse::Ok().body(
            AdminTemplate { links: db_links }
                .render()
                .expect("Failed to render admin template"),
        )
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
