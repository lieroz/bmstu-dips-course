use yaml_rust::{YamlLoader};

pub struct Config {
    app_port: i64,
    app_workers: i64,
    redis_host: String,
    redis_port: i64,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Config, Box<dyn std::error::Error + 'static>> {
        let config = std::fs::read_to_string(config_path)?;
        let yaml = YamlLoader::load_from_str(&config)?;

        Ok(Config{
            app_port: yaml[0]["app"]["port"].as_i64()
                .expect("app port is not present or invalid format"),
            app_workers: yaml[0]["app"]["workers"].as_i64()
                .expect("app workers is not present or invalid format"),
            redis_host: yaml[0]["redis"]["host"].as_str()
                .expect("redis host is not present or invalid format").to_owned(),
            redis_port: yaml[0]["redis"]["port"].as_i64()
                .expect("redis port is not present or invalid format"),
        })
    }

    pub fn get_app_port(&self) -> i64 {
        self.app_port
    }

    pub fn get_app_workers(&self) -> i64 {
        self.app_workers
    }

    pub fn get_redis_host(&self) -> &str {
        &self.redis_host
    }

    pub fn get_redis_port(&self) -> i64 {
        self.redis_port
    }
}
