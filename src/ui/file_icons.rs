// T863-T871: File type icons with emojis
use crate::models::file_entry::{FileEntry, EntryType};
use std::path::Path;

/// Get emoji icon for a file entry based on its type and extension
pub fn get_icon_for_entry(entry: &FileEntry) -> &'static str {
    // T865: Folders
    if entry.is_dir() {
        return if matches!(entry.entry_type, EntryType::Symlink) {
            "🔗" // T865: Symlink to directory
        } else {
            "📁" // T865: Regular directory
        };
    }
    
    // T870: Executables (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(mode) = entry.permissions.as_ref().map(|p| p.mode()) {
            if mode & 0o111 != 0 {
                return "⚡"; // T870: Executable file
            }
        }
    }
    
    // Get file extension
    let extension = Path::new(&entry.name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    
    match extension.as_deref() {
        // T869: Archive files
        Some("zip") | Some("tar") | Some("gz") | Some("7z") | Some("rar") |
        Some("bz2") | Some("xz") | Some("tgz") | Some("tbz2") | Some("txz") => "📦",
        
        // T868: Images
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") |
        Some("webp") | Some("svg") | Some("ico") | Some("tiff") | Some("tif") => "🖼️",
        
        // T868: Audio
        Some("mp3") | Some("wav") | Some("flac") | Some("ogg") | Some("m4a") |
        Some("aac") | Some("wma") | Some("opus") | Some("ape") => "🎵",
        
        // T868: Video
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("wmv") |
        Some("flv") | Some("webm") | Some("m4v") | Some("mpg") | Some("mpeg") => "🎬",
        
        // T866: Documents - text
        Some("txt") | Some("md") | Some("markdown") | Some("rst") => "📄",
        
        // T866: Documents - office
        Some("doc") | Some("docx") | Some("odt") | Some("rtf") => "📝",
        
        // T866: Spreadsheets
        Some("xls") | Some("xlsx") | Some("csv") | Some("ods") => "📊",
        
        // T866: Presentations
        Some("ppt") | Some("pptx") | Some("odp") => "📈",
        
        // T866: PDF
        Some("pdf") => "📕",
        
        // T867: Code files
        Some("rs") | Some("py") | Some("js") | Some("ts") | Some("jsx") | Some("tsx") |
        Some("c") | Some("cpp") | Some("h") | Some("hpp") | Some("java") | Some("go") |
        Some("rb") | Some("php") | Some("swift") | Some("kt") | Some("cs") | Some("scala") => "💻",
        
        // T867: Config files
        Some("json") | Some("yaml") | Some("yml") | Some("toml") | Some("xml") |
        Some("ini") | Some("conf") | Some("cfg") | Some("config") => "⚙️",
        
        // T867: Shell scripts
        Some("sh") | Some("bash") | Some("zsh") | Some("fish") | Some("ksh") |
        Some("csh") | Some("tcsh") | Some("bat") | Some("cmd") | Some("ps1") => "🔧",
        
        // T870: Executables (Windows)
        Some("exe") | Some("msi") | Some("app") | Some("dmg") | Some("deb") |
        Some("rpm") | Some("apk") | Some("bin") => "⚡",
        
        // Web files
        Some("html") | Some("htm") | Some("css") | Some("scss") | Some("sass") |
        Some("less") => "🌐",
        
        // Database files
        Some("db") | Some("sqlite") | Some("sqlite3") | Some("mdb") => "🗄️",
        
        // Font files
        Some("ttf") | Some("otf") | Some("woff") | Some("woff2") | Some("eot") => "🔤",
        
        // Lock files
        Some("lock") => "🔒",
        
        // Log files
        Some("log") => "📋",
        
        // T871: Default for unknown types
        _ => "📄",
    }
}
