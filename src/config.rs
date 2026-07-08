use crate::{ko, ok};
use inquire::{Password, Text};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, create_dir_all},
    io::Write,
};
use unic_langid::LanguageIdentifier;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub database_url: String,
}

pub fn load_config() -> Option<Config> {
    if let Some(config_dir) = dirs::config_dir() {
        let c = config_dir.join("uvd");
        create_dir_all(&c).expect("failed to create config dir");
        let config_path = c.join("config.toml");
        if let Ok(content) = fs::read_to_string(config_path) {
            return toml::from_str(&content).ok();
        }
    }
    None
}

pub fn init_config(lang: &LanguageIdentifier) -> bool {
    if let Some(config_dir) = dirs::config_dir() {
        let c = config_dir.join("uvd");
        create_dir_all(&c).expect("failed to create config dir");
        let config_file = c.join("config.toml");
        let user = Text::new("MySql username:")
            .prompt()
            .expect("username is required");
        let dbname = Text::new("MySql database name:")
            .prompt()
            .expect("dbname is required");
        let port = Text::new("MySql port")
            .with_default("3306")
            .prompt()
            .expect("db port is required");
        let host = Text::new("Host")
            .with_default("localhost")
            .prompt()
            .expect("hostname is required");
        let password = Password::new("Database user password:")
            .prompt()
            .expect("password it's required");
        // Encode l'user et le password pour protéger l'URL des caractères spéciaux
        let encoded_user = urlencoding::encode(&user);
        let encoded_password = urlencoding::encode(&password);

        // Construction sécurisée de l'URL
        let database_url =
            format!("mysql://{encoded_user}:{encoded_password}@{host}:{port}/{dbname}");
        let config = toml::to_string(&Config { database_url }).expect("missingf data");
        let mut file = File::create(config_file).expect("failed to create config file");
        file.write_all(config.as_bytes())
            .expect("failed to write config");
        file.sync_all().expect("failed to sync data");
        ok(lang, "config-file-generated-successfully");
        true
    } else {
        ko(lang, "failed-to-generate-config");
        false
    }
}
