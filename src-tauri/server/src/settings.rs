use std::sync::OnceLock;

use anyhow::Result;
use config::Config;
use serde::Deserialize;

use crate::{file_system, listener::TcpListenerConfig, restful_api::ServerConfig};

#[derive(Deserialize)]
pub struct Settings {
    pub file_system: file_system::FileSystemConfig,
    pub http_server: ServerConfig,
    pub tcp_server: TcpListenerConfig,
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
