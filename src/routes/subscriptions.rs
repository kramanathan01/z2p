use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct User {
    name: String,
    email: String,
}

pub async fn subscribe(_form: web::Form<User>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
