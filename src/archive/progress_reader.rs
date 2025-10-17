// Progress tracking wrapper for reading files during extraction
use std::io::{self, Read};
use std::sync::mpsc::Sender;
use crate::models::operation::Progress;

/// Wrapper around a Read that sends progress updates using std::sync::mpsc
/// This is thread-safe and works from within spawn_blocking contexts
pub struct ProgressReader<R: Read> {
    inner: R,
    progress_tx: Sender<Progress>,
    bytes_read: u64,
    last_update_bytes: u64,
    update_interval: u64,
    // Progress context
    current_file_index: usize,
    total_files: usize,
    bytes_before_this_file: u64,
    total_bytes: u64,
}

impl<R: Read> ProgressReader<R> {
    pub fn new(
        inner: R,
        progress_tx: Sender<Progress>,
        current_file_index: usize,
        total_files: usize,
        bytes_before_this_file: u64,
        total_bytes: u64,
    ) -> Self {
        Self {
            inner,
            progress_tx,
            bytes_read: 0,
            last_update_bytes: 0,
            update_interval: 1024 * 1024, // Update every 1MB
            current_file_index,
            total_files,
            bytes_before_this_file,
            total_bytes,
        }
    }

    fn send_progress(&mut self) {
        let total_bytes_done = self.bytes_before_this_file + self.bytes_read;
        
        // Use std::sync::mpsc which is safe from blocking threads
        let _ = self.progress_tx.send(Progress {
            bytes_done: total_bytes_done,
            bytes_total: self.total_bytes,
            files_done: self.current_file_index,
            files_total: self.total_files,
        });
        
        self.last_update_bytes = self.bytes_read;
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        
        if n > 0 {
            self.bytes_read += n as u64;
            
            // Send progress update if we've read enough bytes since last update
            if self.bytes_read - self.last_update_bytes >= self.update_interval {
                self.send_progress();
            }
        }
        
        Ok(n)
    }
}

impl<R: Read> Drop for ProgressReader<R> {
    fn drop(&mut self) {
        // Send final progress when reader is dropped
        if self.bytes_read > self.last_update_bytes {
            self.send_progress();
        }
    }
}
