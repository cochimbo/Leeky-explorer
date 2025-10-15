// T806-T817: Archive format detection and listing
use anyhow::{Result, Context, bail};
use std::path::Path;
use std::fs::File;
use std::io::Read;

/// T807: Archive format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    ZIP,
    TAR,
    TAR_GZ,
    TAR_BZ2,
    TAR_XZ,
    SEVENZ,
    RAR,
    UNKNOWN,
}

/// T811: Archive entry information
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub size_compressed: u64,
    pub size_uncompressed: u64,
    pub is_dir: bool,
}

/// T808-T810: Detect archive format using magic bytes and extension
pub fn detect_format(path: &Path) -> Result<ArchiveFormat> {
    // Try magic bytes first
    let mut file = File::open(path).context("Failed to open archive")?;
    let mut magic = [0u8; 8];
    let bytes_read = file.read(&mut magic)?;
    
    if bytes_read >= 4 {
        // T809: Check magic byte signatures
        
        // ZIP: PK\x03\x04
        if magic[0..4] == [0x50, 0x4B, 0x03, 0x04] {
            return Ok(ArchiveFormat::ZIP);
        }
        
        // 7Z: 7z\xBC\xAF\x27\x1C
        if bytes_read >= 6 && magic[0..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
            return Ok(ArchiveFormat::SEVENZ);
        }
        
        // RAR: Rar! (0x52 0x61 0x72 0x21)
        if magic[0..4] == [0x52, 0x61, 0x72, 0x21] {
            return Ok(ArchiveFormat::RAR);
        }
        
        // GZIP (for TAR.GZ): \x1F\x8B
        if magic[0..2] == [0x1F, 0x8B] {
            return Ok(ArchiveFormat::TAR_GZ);
        }
        
        // BZ2 (for TAR.BZ2): BZ
        if magic[0..2] == [0x42, 0x5A] {
            return Ok(ArchiveFormat::TAR_BZ2);
        }
        
        // XZ (for TAR.XZ): \xFD7zXZ\x00
        if bytes_read >= 6 && magic[0..6] == [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
            return Ok(ArchiveFormat::TAR_XZ);
        }
        
        // TAR: check for "ustar" at offset 257
        drop(file);
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 262];
        if file.read(&mut buffer)? >= 262 {
            if &buffer[257..262] == b"ustar" {
                return Ok(ArchiveFormat::TAR);
            }
        }
    }
    
    // T810: Fallback to extension detection
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        return Ok(match ext_lower.as_str() {
            "zip" => ArchiveFormat::ZIP,
            "tar" => ArchiveFormat::TAR,
            "gz" | "tgz" => {
                // Check if it's .tar.gz
                if let Some(stem) = path.file_stem() {
                    if stem.to_string_lossy().ends_with(".tar") {
                        ArchiveFormat::TAR_GZ
                    } else {
                        ArchiveFormat::UNKNOWN
                    }
                } else {
                    ArchiveFormat::TAR_GZ
                }
            }
            "bz2" | "tbz" | "tbz2" => ArchiveFormat::TAR_BZ2,
            "xz" | "txz" => ArchiveFormat::TAR_XZ,
            "7z" => ArchiveFormat::SEVENZ,
            "rar" => ArchiveFormat::RAR,
            _ => ArchiveFormat::UNKNOWN,
        });
    }
    
    Ok(ArchiveFormat::UNKNOWN)
}

/// T812-T817: List contents of an archive
pub fn list_archive_contents(path: &Path, format: ArchiveFormat) -> Result<Vec<ArchiveEntry>> {
    match format {
        ArchiveFormat::ZIP => list_zip_contents(path),
        ArchiveFormat::TAR => list_tar_contents(path, None),
        ArchiveFormat::TAR_GZ => list_tar_contents(path, Some(CompressionType::Gzip)),
        ArchiveFormat::TAR_BZ2 => list_tar_contents(path, Some(CompressionType::Bzip2)),
        ArchiveFormat::TAR_XZ => list_tar_contents(path, Some(CompressionType::Xz)),
        ArchiveFormat::SEVENZ => list_7z_contents(path),
        ArchiveFormat::RAR => list_rar_contents(path),
        ArchiveFormat::UNKNOWN => bail!("Unknown archive format"),
    }
}

enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
}

/// T813: List ZIP contents
fn list_zip_contents(path: &Path) -> Result<Vec<ArchiveEntry>> {
    let file = File::open(path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;
    
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        entries.push(ArchiveEntry {
            name: file.name().to_string(),
            size_compressed: file.compressed_size(),
            size_uncompressed: file.size(),
            is_dir: file.is_dir(),
        });
    }
    
    Ok(entries)
}

/// T814: List TAR contents
fn list_tar_contents(path: &Path, compression: Option<CompressionType>) -> Result<Vec<ArchiveEntry>> {
    let file = File::open(path).context("Failed to open TAR file")?;
    
    let mut archive: tar::Archive<Box<dyn Read>> = match compression {
        None => {
            tar::Archive::new(Box::new(file))
        }
        Some(CompressionType::Gzip) => {
            let decoder = flate2::read::GzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Bzip2) => {
            let decoder = bzip2::read::BzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
        Some(CompressionType::Xz) => {
            let decoder = xz2::read::XzDecoder::new(file);
            tar::Archive::new(Box::new(decoder))
        }
    };
    
    let mut entries = Vec::new();
    for entry_result in archive.entries()? {
        let entry = entry_result?;
        let header = entry.header();
        
        entries.push(ArchiveEntry {
            name: entry.path()?.to_string_lossy().to_string(),
            size_compressed: header.size()?, // TAR doesn't store compressed size separately
            size_uncompressed: header.size()?,
            is_dir: header.entry_type().is_dir(),
        });
    }
    
    Ok(entries)
}

/// T815: List 7Z contents
fn list_7z_contents(path: &Path) -> Result<Vec<ArchiveEntry>> {
    let file = File::open(path).context("Failed to open 7Z file")?;
    let len = file.metadata()?.len();
    
    let password: sevenz_rust::Password = "".into(); // No password for listing
    
    let archive = sevenz_rust::SevenZReader::new(file, len, password)
        .context("Failed to read 7Z archive")?;
    
    let mut entries = Vec::new();
    for entry in &archive.archive().files {
        if entry.has_stream() {
            entries.push(ArchiveEntry {
                name: entry.name().to_string(),
                size_compressed: 0, // 7Z doesn't provide per-file compressed size easily
                size_uncompressed: entry.size(),
                is_dir: entry.is_directory(),
            });
        }
    }
    
    Ok(entries)
}

/// T816: List RAR contents (placeholder - requires unrar library)
fn list_rar_contents(_path: &Path) -> Result<Vec<ArchiveEntry>> {
    // Note: RAR support requires external libunrar library
    // For now, return an error indicating RAR is not supported
    bail!("RAR format is not yet supported. Please install libunrar.")
}

/// T817: Calculate compression ratio
pub fn calculate_compression_ratio(entries: &[ArchiveEntry]) -> f64 {
    let total_compressed: u64 = entries.iter().map(|e| e.size_compressed).sum();
    let total_uncompressed: u64 = entries.iter().map(|e| e.size_uncompressed).sum();
    
    if total_uncompressed == 0 {
        return 0.0;
    }
    
    (1.0 - (total_compressed as f64 / total_uncompressed as f64)) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // TODO: Create temporary test files for this test
    fn test_detect_format_by_extension() {
        assert_eq!(
            detect_format(Path::new("test.zip")).unwrap(),
            ArchiveFormat::ZIP
        );
        assert_eq!(
            detect_format(Path::new("test.tar.gz")).unwrap(),
            ArchiveFormat::TAR_GZ
        );
        assert_eq!(
            detect_format(Path::new("test.7z")).unwrap(),
            ArchiveFormat::SEVENZ
        );
    }
    
    #[test]
    fn test_compression_ratio() {
        let entries = vec![
            ArchiveEntry {
                name: "file1.txt".to_string(),
                size_compressed: 50,
                size_uncompressed: 100,
                is_dir: false,
            },
            ArchiveEntry {
                name: "file2.txt".to_string(),
                size_compressed: 50,
                size_uncompressed: 100,
                is_dir: false,
            },
        ];
        
        let ratio = calculate_compression_ratio(&entries);
        assert_eq!(ratio, 50.0); // (1 - 100/200) * 100 = 50%
    }
}
