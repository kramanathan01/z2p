pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

// use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

// use std::net::TcpListener;
// #[derive(serde::Deserialize)]
// struct User {
//     name: String,
//     email: String,
// }

// async fn health_check() -> impl Responder {
//     HttpResponse::Ok().finish()
// }

// async fn subscribe(_form: web::Form<User>) -> impl Responder {
//     HttpResponse::Ok().finish()
// }

// pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
//     let server = HttpServer::new(|| {
//         App::new()
//             .route("/{health_check}", web::get().to(health_check))
//             .route("/{subscribe}", web::post().to(subscribe))
//     })
//     .listen(listener)?
//     .run();
//     Ok(server)
// }
