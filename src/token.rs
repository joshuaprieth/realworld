use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SECRET: &[u8] = b"nuclear launch codes";

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    user_id: i64,
}

pub fn create_token(user_id: i64) -> String {
    let my_claims = Claims {
        exp: 10000000000,
        user_id: user_id,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &my_claims,
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap();

    token
}

pub fn authenticate(token: &str) -> i64 {
    let token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::new(Algorithm::HS256),
    )
    .unwrap();

    token.claims.user_id
}
