use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};

/// Detect the encoding of a byte slice
/// Returns UTF-8 if valid UTF-8, otherwise falls back to Windows-1252 (Latin-1)
pub fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // First, check if it's valid UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        return UTF_8;
    }

    // Fallback to Windows-1252 (compatible with Latin-1)
    WINDOWS_1252
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8() {
        let utf8_text = "Hello, world! 你好世界".as_bytes();
        assert_eq!(detect_encoding(utf8_text), UTF_8);
    }

    #[test]
    fn test_detect_latin1() {
        // Bytes that are invalid UTF-8 but valid Latin-1
        let latin1_bytes = vec![0xC0, 0xE9, 0xF1]; // À é ñ in Latin-1
        assert_eq!(detect_encoding(&latin1_bytes), WINDOWS_1252);
    }
}
