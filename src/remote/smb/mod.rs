// SMB/CIFS connection management
mod manager;

#[cfg(windows)]
mod windows_impl;

#[cfg(unix)]
mod unix_impl;

pub use manager::SmbManager;
