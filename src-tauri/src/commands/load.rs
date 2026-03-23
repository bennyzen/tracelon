// src-tauri/src/commands/load.rs
use std::sync::{Arc, Mutex};
use image::GenericImageView;
use crate::types::ImageInfo;
use crate::AppState;

/// Inner function for testing without Tauri state wrapper.
pub fn load_image_inner(
    state: &Mutex<AppState>,
    path: String,
) -> Result<ImageInfo, String> {
    let img = image::open(&path).map_err(|e| format!("Failed to open image: {e}"))?;
    let (width, height) = img.dimensions();

    // Store in state
    let mut state = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    state.loaded_image = Some(img);
    state.cached_trace_paths = None;
    state.cached_trace_viewbox = None;

    Ok(ImageInfo {
        width,
        height,
        path,
    })
}

#[tauri::command]
pub fn load_image(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    path: String,
) -> Result<ImageInfo, String> {
    load_image_inner(&state, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, ImageBuffer};
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_png() {
        let img: RgbaImage = ImageBuffer::from_pixel(100, 80, image::Rgba([255, 0, 0, 255]));
        let tmp = NamedTempFile::with_suffix(".png").unwrap();
        img.save(tmp.path()).unwrap();

        let state = Mutex::new(AppState::default());
        let info = load_image_inner(&state, tmp.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 80);
        assert!(!info.path.is_empty());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let state = Mutex::new(AppState::default());
        let result = load_image_inner(&state, "/tmp/does_not_exist_12345.png".to_string());
        assert!(result.is_err());
    }
}
