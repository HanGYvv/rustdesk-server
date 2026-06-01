use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: u32,
    pub exp: usize,
}

pub fn secret() -> String {
    std::env::var("RUSTDESK_API_JWT_KEY").unwrap_or_default()
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    let secret = secret();
    if secret.is_empty() {
        return Err("JWT secret is not configured".to_owned());
    }

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|token_data| token_data.claims)
    .map_err(|_| "Invalid token".to_owned())
}
