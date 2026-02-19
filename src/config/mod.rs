use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub postgres_url: String,
    pub rate_limit_requests_per_minute: u32,
    pub server_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists (for local development)
        dotenvy::dotenv().ok();

        let server_address =
            env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let postgres_url = env::var("POSTGRES_URL")
            .expect("POSTGRES_URL must be set (e.g. postgres://user:password@host:port/dbname)");

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .expect("RATE_LIMIT_REQUESTS_PER_MINUTE must be a valid number");

        let server_password = env::var("SERVER_PASSWORD").ok();

        Self {
            server_address,
            postgres_url,
            rate_limit_requests_per_minute,
            server_password,
        }
    }

    pub fn database_url(&self) -> String {
        self.postgres_url.clone()
    }
}
