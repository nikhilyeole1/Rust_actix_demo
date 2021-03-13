//! Actix web Diesel integration example
//!
//! Diesel does not support tokio, so we have to run it in separate threads using the web::block
//! function which offloads blocking code (like Diesel's) in order to not block the server's thread.

#[macro_use]
extern crate diesel;

use actix_web::{ middleware, App,  HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
mod actions;
mod models;
mod order_handlers;
mod user_handlers;
mod schema;
mod token_utils;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";

    println!("Starting server at: {}", &bind);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(user_handlers::add_user)
            .service(user_handlers::get_user)
            .service(order_handlers::get_order)
            .service(order_handlers::create_order)
            .service(order_handlers::get_order_details_for_user)
    })
    .bind(&bind)?
    .run()
    .await
}
