use config::{Config, ConfigError, File};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    // Initialise our configuration reader
    let mut settings = Config::default(); // Config

    // Add configurations values from a field named `configuration`.
    // It will look for any top-level file with an extension
    // that `config` knows how to parse: yaml, json, etc.
    settings.merge(File::with_name("configuration"))?; // &mut Config

    // Deserialize Config into the type specified on function signature
    // Result<Settings, ConfigError>
    settings.try_into() // Result<T, Self::Error>
}
