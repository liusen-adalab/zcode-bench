use std::sync::OnceLock;

use anyhow::Result;
use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub remote_server: RemoteServerConfig,
}

#[derive(Deserialize)]
pub struct RemoteServerConfig {
    pub http: String,
    pub tcp: String,
}

impl RemoteServerConfig {
    pub fn api_load_dir_tree() -> String {
        let host = &get_settings().remote_server.http;
        format!("{}/api/fs/load_structure", host)
    }

    pub fn api_load_dir() -> String {
        let host = &get_settings().remote_server.http;
        format!("{}/api/fs/load_dir", host)
    }
}

#[allow(dead_code)]
mod run_mode {
    pub static RUN_MODE_ENV_KEY: &str = "APP_RUN_MODE";
    pub static RUN_MODE_DEV: &str = "development";
    pub static RUN_MODE_TEST: &str = "test";
    pub static RUN_MODE_PROD: &str = "production";
    pub static RUN_MODE_BENCH: &str = "bench";
    pub static RUN_MODE_BETA: &str = "beta";
}

pub fn load_setttings() -> Result<&'static Settings> {
    #[cfg(test)]
    let run_mode = String::from("test");
    #[cfg(not(test))]
    let run_mode =
        std::env::var(run_mode::RUN_MODE_ENV_KEY).unwrap_or_else(|_| "development".into());

    println!("server running in {} mode", run_mode);

    let settings = Config::builder()
        .add_source(config::File::with_name("configs/default.toml"))
        .add_source(config::File::with_name(&format!(
            "configs/{}.toml",
            run_mode
        )))
        .add_source(config::Environment::with_prefix("AV1_VIDEO"))
        .build()?
        .try_deserialize()?;

    Ok(SETTINGS.get_or_init(|| settings))
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn get_settings() -> &'static Settings {
    SETTINGS.get().unwrap()
}
