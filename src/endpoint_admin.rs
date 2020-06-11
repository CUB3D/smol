use crate::models::Link;
use crate::schema::links::dsl::*;
use crate::DBHandle;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use askama::Template;
use diesel::RunQueryDsl;

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    pub links: Vec<Link>,
}

#[get("/admin")]
pub async fn admin(pool: Data<DBHandle>) -> impl Responder {
    if let Ok(conn) = pool.get() {
        let db_links = links.load::<Link>(&conn).unwrap_or_else(|_| vec![]);
        HttpResponse::Ok().body(AdminTemplate { links: db_links }.render().unwrap())
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
