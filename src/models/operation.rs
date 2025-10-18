// Operation models for file operations
use std::path::PathBuf;

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
        (self.bytes_done as f64 / self.bytes_total as f64) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.bytes_done >= self.bytes_total
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operation_type: OperationType,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub progress: Progress,
    pub batch_items: Option<Vec<(PathBuf, PathBuf, String)>>, // (source, dest, name) for batch ops
    pub current_item_index: usize, // Track which item in batch we're processing
    pub archive_format: Option<crate::archive::formats::ArchiveFormat>, // For extract operations
    pub password: Option<String>, // For password-protected archives
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
