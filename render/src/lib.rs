#![deny(clippy::unwrap_used)]

pub mod backend;
pub mod bitmap;
pub mod bounding_box;
pub mod color_transform;
pub mod error;
pub mod matrix;
pub mod shape_utils;
pub mod transform;
pub mod utils;

pub mod commands;
#[cfg(feature = "tessellator")]
pub mod tessellator;
