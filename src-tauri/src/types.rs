// src-tauri/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TraceMode {
    Monochrome,
    #[serde(rename_all = "camelCase")]
    MultiColor {
        colors: u8,
        cutout: bool,
        filter_speckle: u32,
        color_precision: u8,
    },
    Outline,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineParams {
    pub smoothness: f64,
    pub line_snap: f64,
}

impl PipelineParams {
    pub fn from_smoothness(s: f64) -> Self {
        Self {
            smoothness: s,
            line_snap: 0.5 + s * 2.0,       // 0.5-2.5 px
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgData {
    pub paths: String,
    pub path_count: usize,
    pub segment_count: usize,
    pub raw_segment_count: usize,
    pub viewbox: String,
    pub estimated_size: usize,
}
