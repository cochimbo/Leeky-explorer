// Remote file system support
pub mod vfs;
pub mod sftp;
pub mod connection_manager;
pub mod migration;

pub use vfs::{VirtualFileSystem, VfsEntry, VfsEntryType};
pub use connection_manager::{AuthMethod, ConnectionManager, ConnectionType, ConnectionConfig};
