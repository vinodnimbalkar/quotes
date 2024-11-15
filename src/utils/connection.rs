use mongodb::{
    options::{ClientOptions, Compressor},
    Client, Database,
};
use std::time::Duration;
use tracing::info;

pub struct DatabaseConfig {
    pub uri: String,
    pub connection_timeout: Option<Duration>,
    pub min_pool_size: Option<u32>,
    pub max_pool_size: Option<u32>,
    pub compressors: Option<Vec<Compressor>>,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        let mongo_uri: String = std::env::var("MONGO_URI")
            .expect("Failed to load `MONGO_MAX_POOL_SIZE` environment variable.");

        let mongo_connection_timeout: u64 = std::env::var("MONGO_CONNECTION_TIMEOUT")
            .expect("Failed to load `MONGO_CONNECTION_TIMEOUT` environment variable.")
            .parse()
            .expect("Failed to parse `MONGO_CONNECTION_TIMEOUT` environment variable.");

        let mongo_min_pool_size: u32 = std::env::var("MONGO_MIN_POOL_SIZE")
            .expect("Failed to load `MONGO_MIN_POOL_SIZE` environment variable.")
            .parse()
            .expect("Failed to parse `MONGO_MIN_POOL_SIZE` environment variable.");

        let mongo_max_pool_size: u32 = std::env::var("MONGO_MAX_POOL_SIZE")
            .expect("Failed to load `MONGO_MAX_POOL_SIZE` environment variable.")
            .parse()
            .expect("Failed to parse `MONGO_MAX_POOL_SIZE` environment variable.");

        Self {
            uri: mongo_uri,
            connection_timeout: Some(Duration::from_secs(mongo_connection_timeout)),
            min_pool_size: Some(mongo_min_pool_size),
            max_pool_size: Some(mongo_max_pool_size),
            compressors: Some(vec![
                Compressor::Snappy,
                Compressor::Zlib {
                    level: Default::default(),
                },
                Compressor::Zstd {
                    level: Default::default(),
                },
            ]),
        }
    }
}

pub async fn dbconnect() -> mongodb::error::Result<(Client, Database)> {
    let database_config = DatabaseConfig::new();
    let mut client_options = ClientOptions::parse(database_config.uri).await.unwrap();
    client_options.connect_timeout = database_config.connection_timeout;
    client_options.max_pool_size = database_config.max_pool_size;
    client_options.min_pool_size = database_config.min_pool_size;
    // the server will select the algorithm it supports from the list provided by the driver
    client_options.compressors = database_config.compressors;
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("vinod");

    info!("Connected to database");
    Ok((client, db))
}
