use chrono::Utc;
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;

static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds
// Todo: To load this secret from environment variable.
static KEY: &[u8] = b"some_secret_key";

#[derive(Serialize, Deserialize)]
struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // data
    pub user_id: uuid::Uuid,
}

/// Create jwt token by making use of user id.
pub fn generate_jwt(uid: uuid::Uuid) -> String {
    let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
    let payload = UserToken {
        iat: now,
        exp: now + ONE_WEEK,
        user_id: uid,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(KEY),
    )
    .unwrap()
}


pub fn decode_jwt_and_get_user_id(token: &str) -> Result<uuid::Uuid, Box<dyn Error>> {
    let token_data = jsonwebtoken::decode::<UserToken>(token, &DecodingKey::from_secret(&KEY), &Validation::default())?;
    Ok(token_data.claims.user_id)
}
