//! Actix web Diesel integration example
//!
//! Diesel does not support tokio, so we have to run it in separate threads using the web::block
//! function which offloads blocking code (like Diesel's) in order to not block the server's thread.

use crate::actions;
use crate::models;
use actix_web::error::{ErrorNotFound, ErrorInternalServerError, ErrorUnauthorized, BlockingError};
use actix_web::http::StatusCode;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Inserts new user with name defined in form.
#[post("/api/v1/orders")]
pub async fn create_order(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    form: web::Json<models::NewOrder>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let order_id = Uuid::new_v4();
    //let user_id = Uuid::parse_str("a16aec39-1668-4d4b-a5dd-4488093acc7b").unwrap();
    let note_option = form.note.clone();

    let jwt_header = req.headers().get("access_token").cloned();


    // use web::block to offload blocking Diesel code without blocking server thread
    let order = web::block(move || {
        let user_id = actions::authenticate_request(jwt_header, &conn)?;
        actions::insert_new_order(order_id.clone(), user_id, note_option, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("Print error {}", e);
        match e {
            BlockingError::Error(StatusCode::UNAUTHORIZED) => ErrorUnauthorized("Provide proper access token"),
            BlockingError::Error(StatusCode::NOT_FOUND) => ErrorNotFound("User could not be found to create new order."),
            _ => actix_web::error::ErrorInternalServerError("Something unexpected happened. Please retry"),
        }
    })?;

    let conn2 = pool.get().expect("couldn't get db connection from pool");

    web::block(move || actions::insert_new_order_items(order_id, form.items.clone(), &conn2))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(order))
}

#[get("/api/v1/orders")]
pub async fn get_order_details_for_user(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let order_details = web::block(move || actions::find_all_orders(&conn))
        .await
        .map_err(|e| { ErrorInternalServerError("Something unexpected happened. Please retry")})?;

        Ok(HttpResponse::Ok().json(order_details))
    
}

/// Finds user by UID.
#[get("/api/v1/orders/{order_id}")]
pub async fn get_order(
    pool: web::Data<DbPool>,
    order_uid: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let user_uid = order_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let order = web::block(move || actions::find_order_by_uid(user_uid, &conn))
        .await
        .map_err(|e| {
            match e {
                BlockingError::Error(StatusCode::NOT_FOUND) => ErrorNotFound("Order id not correct."),
                _ => ErrorInternalServerError("Something unexpected happened. Please retry"),
            }
        })?;

    Ok(HttpResponse::Ok().json(order))
}
