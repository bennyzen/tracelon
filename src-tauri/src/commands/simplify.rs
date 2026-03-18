// src-tauri/src/commands/simplify.rs
use std::sync::Mutex;
use crate::types::{SvgData, PipelineParams};
use crate::AppState;
use crate::commands::trace::apply_simplification;

#[tauri::command]
pub fn simplify(state: tauri::State<'_, Mutex<AppState>>, params: PipelineParams) -> Result<SvgData, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    let paths = app.cached_trace_paths.as_ref().ok_or("No trace cached — run trace first")?.clone();
    let viewbox = app.cached_trace_viewbox.as_ref().ok_or("No trace cached")?.clone();
    drop(app);
    apply_simplification(&paths, &viewbox, &params)
}
