// src-tauri/src/lib.rs
mod types;
mod commands;
mod pipeline;

use std::sync::Mutex;
use image::DynamicImage;

pub struct AppState {
    pub loaded_image: Option<DynamicImage>,
    pub cached_trace_paths: Option<Vec<String>>,
    pub cached_trace_viewbox: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_image: None,
            cached_trace_paths: None,
            cached_trace_viewbox: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::load::load_image,
            commands::trace::trace,
            commands::simplify::simplify,
            commands::export::export_svg,
            commands::export::export_optimized_svg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
