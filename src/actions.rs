use actix_web::http::{HeaderValue, StatusCode};
use diesel::prelude::*;
use models::NewOrderItem;
use models::{Order, OrderDetails, OrderItem, OrderItemDetails};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{models, token_utils};

/// Run query using Diesel to find user by uid and return it.
pub fn find_user_by_uid(
    uid: Uuid,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(user_id.eq(uid))
        .first::<models::User>(conn)
        .optional()?;

    Ok(user)
}

pub fn find_user_by_email(
    email_str: &str,
    conn: &PgConnection,
) -> Result<Option<models::User>, StatusCode> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(email.eq(email_str))
        .first::<models::User>(conn)
        .optional()
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(user)
}

pub fn authenticate_request(
    header: Option<HeaderValue>,
    conn: &PgConnection,
) -> Result<uuid::Uuid, StatusCode> {
    // let jwt_token = header.unwrap().to_str().unwrap().into();

    let v = header.ok_or(StatusCode::UNAUTHORIZED)?;
    let jwt_str = v.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id =
        token_utils::decode_jwt_and_get_user_id(jwt_str).map_err(|e| StatusCode::UNAUTHORIZED)?;

    let user_option =
        find_user_by_uid(user_id, conn).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    user_option.map(|u| u.user_id).ok_or(StatusCode::NOT_FOUND)
}

pub fn find_order_by_id(user_id_arg: Uuid, oid: Uuid, conn: &PgConnection) -> Result<OrderDetails, StatusCode> {
    use crate::schema::orders::dsl::*;
    use crate::schema::order_items::dsl::*;

    let vec: Vec<(Order, OrderItem)> = orders
        .inner_join(order_items)
        .filter(crate::schema::orders::dsl::user_id.eq(user_id_arg))
        .filter(crate::schema::orders::dsl::order_id.eq(oid))
        .into_boxed()
        .get_results(conn)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    if vec.len() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    let order = vec[0].0.clone();

    let mut ret_value: OrderDetails = OrderDetails {
        order_id: order.order_id,
        user_id: order.user_id,
        note: order.note,
        order_total: 0,
        order_at: order.created_at,
        items: None, //vec![]
    };

    let mut order_total: i64 = 0;
    let mut order_item_details_vec: Vec<OrderItemDetails> = vec![];

    vec.iter().for_each(|tup| {
        let order_item = tup.1.clone();
        order_total = order_total + i64::from(order_item.qty * order_item.price);

        order_item_details_vec.push(OrderItemDetails {
            item_id: order_item.item_id,
            description: order_item.description,
            qty: order_item.qty,
            price: order_item.price,
        });
    });

    ret_value.order_total = order_total;
    ret_value.items = Some(order_item_details_vec);

    println!(
        "returned value:=======================================> {:?}",
        ret_value
    );

    Ok(ret_value)
}

pub fn find_all_orders(uid: Uuid, conn: &PgConnection) -> Result<Vec<OrderDetails>, StatusCode> {
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;

    let vec: Vec<(Order, OrderItem)> = orders
        .inner_join(order_items)
        .filter(user_id.eq(uid))
        .into_boxed()
        .get_results(conn)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut dictionary: HashMap<&uuid::Uuid, OrderDetails> = HashMap::new();

    vec.iter().for_each(|tup| {
        let order = &tup.0;
        let order_item = &tup.1;
        match dictionary.get_mut(&order.order_id) {
            None => dictionary.insert(
                &order.order_id,
                OrderDetails {
                    order_id: order.order_id,
                    user_id: order.user_id,
                    note: order.note.clone(),
                    order_total: i64::from(order_item.qty * order_item.price),
                    order_at: order.created_at,
                    items: None,
                },
            ),
            Some(od) => {
                od.order_total = od.order_total + i64::from(order_item.qty + order_item.price);
                None
            }
        };
    });

    let vec_of_order_details: Vec<OrderDetails> = dictionary.values().cloned().collect();

    Ok(vec_of_order_details)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    // prevent collision with `name` column imported inside the function
    first_n: &str,
    last_n: &str,
    email_str: &str,
    passwd: &str,
    conn: &PgConnection,
) -> Result<models::User, StatusCode> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::users::dsl::*;

    let new_user = models::User {
        user_id: Uuid::new_v4(),
        first_name: first_n.to_owned(),
        last_name: last_n.to_owned(),
        email: email_str.to_owned(),
        password: passwd.to_owned(),
        created_at: chrono::offset::Utc::now().naive_utc(),
    };

    diesel::insert_into(users).values(&new_user).execute(conn).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(new_user)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_order(
    // prevent collision with `name` column imported inside the function
    order_id_arg: uuid::Uuid,
    user_id_arg: uuid::Uuid,
    note_arg: Option<String>,
    conn: &PgConnection,
) -> Result<models::Order, StatusCode> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::orders::dsl::*;

    let new_order = models::Order {
        order_id: order_id_arg,
        user_id: user_id_arg,
        note: note_arg,
        created_at: chrono::offset::Utc::now().naive_utc(),
    };

    diesel::insert_into(orders)
        .values(&new_order)
        .execute(conn)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(new_order)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_order_items(
    order_id_arg: uuid::Uuid,
    order_items_arg: Vec<NewOrderItem>,
    conn: &PgConnection,
) -> Result<bool, StatusCode> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::order_items::dsl::*;

    let new_order_items: Vec<OrderItem> = order_items_arg
        .iter()
        .map(|oi| OrderItem {
            item_id: Uuid::new_v4(),
            order_id: order_id_arg,
            description: oi.description.clone(),
            qty: oi.qty,
            price: oi.price,
            created_at: chrono::offset::Utc::now().naive_utc(),
        })
        .collect::<Vec<_>>();

    diesel::insert_into(order_items)
        .values(&new_order_items)
        .execute(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(true)
}
