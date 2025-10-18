use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
//use actix_identity::Identity;

use tera::Context;

use crate::models::Nation;
use crate::AppData;
use crate::database::PostgresPool;

#[get("/")]
pub async fn index(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("index.html", &ctx).unwrap();
    // Explicitly set the content type to text/html
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/org_chart")]
pub async fn org_chart(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new(); 
    let rendered = data.tmpl.render("org_chart.html", &ctx).unwrap();
    // Explicitly set the content type to text/html
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/dashboard")]
pub async fn dashboard(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let ctx = Context::new();
    let rendered = data.tmpl.render("dashboard.html", &ctx).unwrap();
    // Explicitly set the content type to text/html
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/nation-analytics")]
pub async fn nation_analytics(data: web::Data<AppData>, _req:HttpRequest) -> impl Responder {
    let mut ctx = Context::new();

    let nation_codes = Nation::get_all_codes()
        .expect("Unable to retrieve nation codes");

    ctx.insert("nations", &nation_codes);

    let rendered = data.tmpl.render("nation_analytics.html", &ctx).unwrap();
    // Explicitly set the content type to text/html
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

#[get("/{lang}/api")]
pub async fn api_base(
    data: web::Data<AppData>,
    _pool: web::Data<PostgresPool>,
    _lang: web::Path<String>,
    _req: HttpRequest,
    // id: Identity,
) -> impl Responder {

    let ctx = Context::new();
    let rendered = data.tmpl.render("api_base.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}


