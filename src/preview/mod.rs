pub mod encoding;
pub mod text_viewer;

pub use encoding::detect_encoding;
pub use text_viewer::{is_text_file, load_text_file};
