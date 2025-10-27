// SMB/CIFS connection management
mod manager;
mod filesystem;

#[cfg(windows)]
mod windows_impl;

#[cfg(unix)]
mod unix_impl;

pub use manager::SmbManager;
pub use filesystem::SmbFileSystem;
