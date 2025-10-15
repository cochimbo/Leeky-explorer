use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

use super::encoding::detect_encoding;

const MAX_PREVIEW_SIZE: u64 = 10 * 1024 * 1024; // 10 MB hard limit
const WARN_PREVIEW_SIZE: u64 = 5 * 1024 * 1024; // 5 MB warning threshold

/// Check if a file is a text file based on extension
pub fn is_text_file(path: &Path) -> bool {
    let text_extensions = [
        "txt", "md", "rs", "py", "js", "ts", "jsx", "tsx", "json", "xml", "html", "css", "scss",
        "log", "conf", "cfg", "ini", "toml", "yaml", "yml", "sh", "bash", "zsh", "fish",
        "c", "cpp", "h", "hpp", "java", "go", "rb", "php", "swift", "kt", "sql", "csv",
        "gitignore", "gitattributes", "dockerignore", "env",
    ];

    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return text_extensions.contains(&ext_str.to_lowercase().as_str());
        }
    }

    // Check for files without extension that are typically text
    if let Some(filename) = path.file_name() {
        if let Some(name_str) = filename.to_str() {
            let common_text_files = [
                "README", "LICENSE", "Makefile", "Dockerfile", "Cargo.lock",
                ".gitignore", ".gitattributes", ".dockerignore", ".env",
            ];
            return common_text_files.iter().any(|&name| {
                name_str.eq_ignore_ascii_case(name) || name_str.starts_with(name)
            });
        }
    }

    false
}

/// Check if content is binary (contains too many non-printable characters)
fn is_binary_content(bytes: &[u8]) -> bool {
    const SAMPLE_SIZE: usize = 8192;
    let sample = if bytes.len() > SAMPLE_SIZE {
        &bytes[..SAMPLE_SIZE]
    } else {
        bytes
    };

    let non_printable_count = sample
        .iter()
        .filter(|&&b| {
            // Consider null bytes and other control chars as non-printable
            // Allow: tab (9), newline (10), carriage return (13), and printable ASCII (32-126)
            b != 9 && b != 10 && b != 13 && (b < 32 || b > 126) && b < 128
        })
        .count();

    // If more than 10% is non-printable, consider it binary
    non_printable_count > sample.len() / 10
}

/// Load a text file with encoding detection
/// Returns an error if the file is too large or appears to be binary
pub async fn load_text_file(path: &Path) -> Result<(String, Option<String>)> {
    // Check file size
    let metadata = fs::metadata(path)
        .await
        .context("Failed to read file metadata")?;
    let file_size = metadata.len();

    if file_size > MAX_PREVIEW_SIZE {
        anyhow::bail!(
            "File too large for preview: {} MB (max: {} MB)",
            file_size / (1024 * 1024),
            MAX_PREVIEW_SIZE / (1024 * 1024)
        );
    }

    let warning = if file_size > WARN_PREVIEW_SIZE {
        Some(format!(
            "Large file ({} MB) - loading may take a moment",
            file_size / (1024 * 1024)
        ))
    } else {
        None
    };

    // Read file bytes
    let bytes = fs::read(path).await.context("Failed to read file")?;

    // Check if binary
    if is_binary_content(&bytes) {
        anyhow::bail!("Cannot preview: file appears to be binary");
    }

    // Detect encoding and decode
    let encoding = detect_encoding(&bytes);
    let (content, _, had_errors) = encoding.decode(&bytes);

    if had_errors {
        log::warn!("Encoding errors detected while reading {:?}", path);
    }

    Ok((content.into_owned(), warning))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(Path::new("test.txt")));
        assert!(is_text_file(Path::new("test.rs")));
        assert!(is_text_file(Path::new("test.md")));
        assert!(is_text_file(Path::new("test.json")));
        assert!(is_text_file(Path::new("README")));
        assert!(is_text_file(Path::new("Makefile")));

        assert!(!is_text_file(Path::new("test.png")));
        assert!(!is_text_file(Path::new("test.jpg")));
        assert!(!is_text_file(Path::new("test.exe")));
        assert!(!is_text_file(Path::new("test.bin")));
    }

    #[test]
    fn test_is_binary_content() {
        let text = b"Hello, world!\nThis is a text file.";
        assert!(!is_binary_content(text));

        let binary = vec![0u8; 1000]; // Null bytes
        assert!(is_binary_content(&binary));

        let mixed = {
            let mut v = Vec::new();
            v.extend_from_slice(b"Some text");
            v.extend_from_slice(&[0u8; 200]); // Many nulls
            v
        };
        assert!(is_binary_content(&mixed));
    }

    #[tokio::test]
    async fn test_load_utf8_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Hello, world!\n你好世界\n";
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let (loaded_content, warning) = load_text_file(temp_file.path()).await.unwrap();
        assert_eq!(loaded_content, content);
        assert!(warning.is_none());
    }

    #[tokio::test]
    async fn test_load_latin1_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write bytes that are invalid UTF-8 but valid Latin-1
        let latin1_bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xC0, 0xE9, 0xF1]; // "Hello Àéñ"
        temp_file.write_all(&latin1_bytes).unwrap();
        temp_file.flush().unwrap();

        let (loaded_content, _) = load_text_file(temp_file.path()).await.unwrap();
        assert!(loaded_content.contains("Hello"));
        // Should have decoded the Latin-1 characters
        assert!(loaded_content.len() > 5);
    }

    #[tokio::test]
    async fn test_reject_binary_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write binary content (null bytes)
        let binary_content = vec![0u8; 1000];
        temp_file.write_all(&binary_content).unwrap();
        temp_file.flush().unwrap();

        let result = load_text_file(temp_file.path()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("appears to be binary"));
    }

    #[tokio::test]
    async fn test_file_size_warning() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Create a file just over 5MB
        let large_content = "x".repeat(6 * 1024 * 1024);
        temp_file.write_all(large_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let (_, warning) = load_text_file(temp_file.path()).await.unwrap();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Large file"));
    }
}
