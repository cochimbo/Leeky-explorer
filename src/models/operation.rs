// Operation models for file operations
use std::path::PathBuf;
use std::sync::Arc;
use crate::remote::VirtualFileSystem;

#[derive(Debug, Clone)]
pub enum OperationType {
    Copy,
    Move,
    Delete,
    Extract,
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub files_done: usize,
    pub files_total: usize,
}

impl Progress {
    pub fn new(bytes_total: u64, files_total: usize) -> Self {
        Self {
            bytes_done: 0,
            bytes_total,
            files_done: 0,
            files_total,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.bytes_total == 0 {
            return 0.0;
        }
        let percentage = (self.bytes_done as f64 / self.bytes_total as f64) * 100.0;
        // Clamp to 0-100 range to prevent panic in gauge widget
    percentage.clamp(0.0, 100.0)
    }

    pub fn is_complete(&self) -> bool {
        self.bytes_done >= self.bytes_total
    }
}

#[derive(Clone)]
pub struct Operation {
    pub operation_type: OperationType,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub progress: Progress,
    pub batch_items: Option<Vec<(PathBuf, PathBuf, String)>>, // (source, dest, name) for batch ops
    pub current_item_index: usize, // Track which item in batch we're processing
    pub archive_format: Option<crate::archive::formats::ArchiveFormat>, // For extract operations
    pub password: Option<String>, // For password-protected archives
    // Remote filesystem support
    pub source_vfs: Option<Arc<dyn VirtualFileSystem>>, // None = local, Some = remote
    pub dest_vfs: Option<Arc<dyn VirtualFileSystem>>,   // None = local, Some = remote
}

// Manual Debug impl since VirtualFileSystem doesn't impl Debug
impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Operation")
            .field("operation_type", &self.operation_type)
            .field("source", &self.source)
            .field("destination", &self.destination)
            .field("progress", &self.progress)
            .field("batch_items", &self.batch_items)
            .field("current_item_index", &self.current_item_index)
            .field("archive_format", &self.archive_format)
            .field("password", &"<redacted>")
            .field("source_vfs", &self.source_vfs.as_ref().map(|_| "Some(VFS)"))
            .field("dest_vfs", &self.dest_vfs.as_ref().map(|_| "Some(VFS)"))
            .finish()
    }
}

impl Operation {
    pub fn copy(source: PathBuf, destination: PathBuf, total_bytes: u64, total_files: usize) -> Self {
        Self {
            operation_type: OperationType::Copy,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn copy_vfs(
        source: PathBuf,
        destination: PathBuf,
        total_bytes: u64,
        total_files: usize,
        source_vfs: Option<Arc<dyn VirtualFileSystem>>,
        dest_vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        Self {
            operation_type: OperationType::Copy,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs,
            dest_vfs,
        }
    }

    pub fn move_op(source: PathBuf, destination: PathBuf, total_bytes: u64, total_files: usize) -> Self {
        Self {
            operation_type: OperationType::Move,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn move_vfs(
        source: PathBuf,
        destination: PathBuf,
        total_bytes: u64,
        total_files: usize,
        source_vfs: Option<Arc<dyn VirtualFileSystem>>,
        dest_vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        Self {
            operation_type: OperationType::Move,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs,
            dest_vfs,
        }
    }

    pub fn delete(path: PathBuf, total_bytes: u64, total_files: usize) -> Self {
        Self {
            operation_type: OperationType::Delete,
            source: path.clone(),
            destination: path,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn delete_vfs(
        path: PathBuf,
        total_bytes: u64,
        total_files: usize,
        vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        Self {
            operation_type: OperationType::Delete,
            source: path.clone(),
            destination: path,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: vfs.clone(),
            dest_vfs: vfs,  // For delete, both are the same
        }
    }

    pub fn extract(
        source: PathBuf,
        destination: PathBuf,
        total_bytes: u64,
        total_files: usize,
        format: crate::archive::formats::ArchiveFormat,
    ) -> Self {
        Self {
            operation_type: OperationType::Extract,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: Some(format),
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn extract_with_password(
        source: PathBuf,
        destination: PathBuf,
        total_bytes: u64,
        total_files: usize,
        format: crate::archive::formats::ArchiveFormat,
        password: String,
    ) -> Self {
        Self {
            operation_type: OperationType::Extract,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: None,
            current_item_index: 0,
            archive_format: Some(format),
            password: Some(password),
            source_vfs: None,
            dest_vfs: None,
        }
    }

    // T574: Batch operation constructors
    pub fn copy_batch(items: Vec<(PathBuf, PathBuf, String)>, total_bytes: u64, total_files: usize) -> Self {
        let (source, destination, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Copy,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn copy_batch_vfs(
        items: Vec<(PathBuf, PathBuf, String)>,
        total_bytes: u64,
        total_files: usize,
        source_vfs: Option<Arc<dyn VirtualFileSystem>>,
        dest_vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        let (source, destination, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Copy,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs,
            dest_vfs,
        }
    }

    pub fn move_batch(items: Vec<(PathBuf, PathBuf, String)>, total_bytes: u64, total_files: usize) -> Self {
        let (source, destination, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Move,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn move_batch_vfs(
        items: Vec<(PathBuf, PathBuf, String)>,
        total_bytes: u64,
        total_files: usize,
        source_vfs: Option<Arc<dyn VirtualFileSystem>>,
        dest_vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        let (source, destination, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Move,
            source,
            destination,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs,
            dest_vfs,
        }
    }

    pub fn delete_batch(items: Vec<(PathBuf, PathBuf, String)>, total_bytes: u64, total_files: usize) -> Self {
        let (source, _, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Delete,
            source: source.clone(),
            destination: source,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: None,
            dest_vfs: None,
        }
    }
    
    pub fn delete_batch_vfs(
        items: Vec<(PathBuf, PathBuf, String)>,
        total_bytes: u64,
        total_files: usize,
        vfs: Option<Arc<dyn VirtualFileSystem>>,
    ) -> Self {
        let (source, _, _) = items.first()
            .cloned()
            .unwrap_or_else(|| (PathBuf::new(), PathBuf::new(), String::new()));
        Self {
            operation_type: OperationType::Delete,
            source: source.clone(),
            destination: source,
            progress: Progress::new(total_bytes, total_files),
            batch_items: Some(items),
            current_item_index: 0,
            archive_format: None,
            password: None,
            source_vfs: vfs.clone(),
            dest_vfs: vfs,  // For delete, both are the same
        }
    }

    pub fn is_batch(&self) -> bool {
        self.batch_items.is_some()
    }

    pub fn get_current_item_name(&self) -> Option<String> {
        self.batch_items.as_ref()
            .and_then(|items| items.get(self.current_item_index))
            .map(|(_, _, name)| name.clone())
    }
}
