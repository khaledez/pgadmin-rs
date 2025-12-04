use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub rate_limit_requests_per_minute: u32,
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists (for local development)
        dotenvy::dotenv().ok();

        let server_address =
            env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let postgres_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());

        let postgres_port = env::var("POSTGRES_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .expect("POSTGRES_PORT must be a valid number");

        let postgres_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());

        let postgres_password =
            env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());

        let postgres_db = env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".to_string());

        let rate_limit_requests_per_minute = env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .expect("RATE_LIMIT_REQUESTS_PER_MINUTE must be a valid number");

        Self {
            server_address,
            postgres_host,
            postgres_port,
            postgres_user,
            postgres_password,
            postgres_db,
            rate_limit_requests_per_minute,
        }
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }
}
