// src-tauri/src/commands/export.rs
use std::fs;

#[tauri::command]
pub fn export_svg(svg_data: String, viewbox: String, output_path: String) -> Result<(), String> {
    let svg_doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}">
{svg_data}
</svg>"#
    );
    fs::write(&output_path, svg_doc).map_err(|e| format!("Failed to write SVG: {e}"))
}

/// Export a pre-optimized SVG string directly to disk
#[tauri::command]
pub fn export_optimized_svg(svg_content: String, output_path: String) -> Result<(), String> {
    fs::write(&output_path, svg_content).map_err(|e| format!("Failed to write SVG: {e}"))
}
