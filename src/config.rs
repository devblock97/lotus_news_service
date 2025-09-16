use std::net::SocketAddr;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse().expect("invalid BIND_ADDR");
        Self { database_url: database_url, jwt_secret: jwt_secret, bind_addr: bind_addr}
    }
}