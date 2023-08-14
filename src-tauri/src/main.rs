// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{Duration, Instant};

use tauri::{Manager, Runtime};
use tracing::Level;

use crate::file_system::load_dir_tree;
use crate::file_system::upload_file;

pub mod client;
pub mod file_system;
pub mod my_err;
pub mod settings;
pub mod utils;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn hello_event<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
) -> Result<(), String> {
    println!("sleeping");
    let now = Instant::now();
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("emiting. elpased = {:?}", now.elapsed());
    app.emit_all("hello", "world").unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("emiting window. elpased = {:?}", now.elapsed());
    window.emit("hello", "world from window").unwrap();

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                // window.close_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            hello_event,
            load_dir_tree,
            upload_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
