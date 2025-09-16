use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, Algorithm, encode, decode};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use uuid::Uuid;


#[derive(Clone)]
pub struct JwtKeys {
    pub enc: EncodingKey, pub dec: DecodingKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self {
            enc: EncodingKey::from_secret(secret.as_bytes()),
            dec: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn issue(&self, user_id: Uuid, days: i64) -> anyhow::Result<String> {
        let exp = (Utc::now() + Duration::days(days)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        Ok(encode(&Header::new(Algorithm::HS256), &claims, &self.enc)?)
    }

    pub fn verify(&self, token: &str) -> anyhow::Result<Uuid> {
        let data = decode::<Claims>(token, &self.dec, &Validation::new(Algorithm::HS256))?;
        Ok(Uuid::parse_str(&data.claims.sub)?)
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}