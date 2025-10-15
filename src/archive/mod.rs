// T805: Archive extraction module
pub mod formats;
pub mod extractor;
pub mod password;

pub use formats::{ArchiveFormat, ArchiveEntry, detect_format, list_archive_contents};
pub use extractor::extract_archive;
pub use password::{prompt_password, PasswordDialog};
