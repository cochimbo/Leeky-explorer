pub mod encoding;
pub mod text_viewer;
pub mod image_viewer;

pub use encoding::detect_encoding;
pub use text_viewer::{is_text_file, load_text_file};
pub use image_viewer::{is_image_file, load_image, get_image_metadata, image_to_ascii, ImageMetadata};
