use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

/// Metadata about an image
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
}

/// Check if a file is an image based on extension
pub fn is_image_file(path: &Path) -> bool {
    let image_extensions = [
        "png", "jpg", "jpeg", "gif", "bmp", "webp", "ico", "tiff", "tif",
    ];

    if let Some(extension) = path.extension()
        && let Some(ext_str) = extension.to_str() {
            return image_extensions.contains(&ext_str.to_lowercase().as_str());
        }

    false
}

/// Load an image from a file
pub async fn load_image(path: &Path) -> Result<DynamicImage> {
    let bytes = tokio::fs::read(path)
        .await
        .context("Failed to read image file")?;

    let img = image::load_from_memory(&bytes)
        .context("Failed to decode image")?;

    Ok(img)
}

/// Get image metadata
pub fn get_image_metadata(img: &DynamicImage, path: &Path) -> ImageMetadata {
    let format = detect_format_from_path(path)
        .unwrap_or_else(|| "Unknown".to_string());

    ImageMetadata {
        width: img.width(),
        height: img.height(),
        format,
    }
}

/// Detect image format from file extension
fn detect_format_from_path(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_uppercase())
}

/// Convert image to ASCII/Unicode art with better scaling
/// max_width and max_height are in terminal characters
pub fn image_to_ascii(img: &DynamicImage, max_width: u32, max_height: u32) -> Result<String> {
    // Don't reduce further - the caller already calculated the available space
    // Use full available space for maximum image size
    let effective_width = max_width;
    
    // Since we use half-blocks (2 vertical pixels per character),
    // multiply height by 2 to use actual pixel space
    let effective_height = max_height * 2;
    
    // Calculate scaling to fit within max dimensions while preserving aspect ratio
    let (target_width, target_height) = calculate_target_size(
        img.width(),
        img.height(),
        effective_width,
        effective_height,
    );

    // Resize the image to target size for better quality
    let resized = img.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );

    // Use manual conversion with half-blocks for better quality
    let ascii_art = image_to_ascii_manual(&resized);

    Ok(ascii_art)
}

/// Manual ASCII conversion with half-blocks and ANSI colors for better quality
fn image_to_ascii_manual(img: &DynamicImage) -> String {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    let mut result = String::new();
    
    // Use half-block characters for 2x vertical resolution
    // Process 2 pixels vertically at a time
    for y in (0..height).step_by(2) {
        for x in 0..width {
            let top_pixel = rgb_img.get_pixel(x, y);
            let bottom_pixel = if y + 1 < height {
                rgb_img.get_pixel(x, y + 1)
            } else {
                top_pixel
            };
            
            let top_brightness = calculate_brightness(top_pixel);
            let bottom_brightness = calculate_brightness(bottom_pixel);
            
            // Use half-block characters for better quality
            let char = select_half_block(top_brightness, bottom_brightness);
            
            // Calculate average color for this character
            let rgb = average_rgb(top_pixel, bottom_pixel);
            
            // Add ANSI color code if not a space
            if char != ' ' {
                result.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", rgb.0, rgb.1, rgb.2, char));
            } else {
                result.push(char);
            }
        }
        result.push('\n');
    }
    
    result
}

/// Calculate average RGB color from two pixels
fn average_rgb(pixel1: &image::Rgb<u8>, pixel2: &image::Rgb<u8>) -> (u8, u8, u8) {
    let r = ((pixel1[0] as u16 + pixel2[0] as u16) / 2) as u8;
    let g = ((pixel1[1] as u16 + pixel2[1] as u16) / 2) as u8;
    let b = ((pixel1[2] as u16 + pixel2[2] as u16) / 2) as u8;
    (r, g, b)
}

/// Calculate brightness of a pixel (0-255)
fn calculate_brightness(pixel: &image::Rgb<u8>) -> u8 {
    // Weighted average for perceived brightness
    let r = pixel[0] as f32;
    let g = pixel[1] as f32;
    let b = pixel[2] as f32;
    
    (0.299 * r + 0.587 * g + 0.114 * b) as u8
}

/// Select half-block character based on top and bottom brightness
fn select_half_block(top: u8, bottom: u8) -> char {
    // Use adjusted threshold levels with higher minimum to avoid noise in light areas
    const LEVELS: [u8; 4] = [32, 96, 160, 220];
    
    // Determine brightness level (0-3) for each pixel
    let top_level = LEVELS.iter().position(|&l| top < l).unwrap_or(3);
    let bottom_level = LEVELS.iter().position(|&l| bottom < l).unwrap_or(3);
    
    // Use half-blocks for better representation, avoid light shade characters
    match (top_level, bottom_level) {
        // Both very bright - full block
        (3, 3) => '█',
        // Top brighter than bottom
        (3, 2) => '▀',
        (3, 1) => '▀',
        (3, 0) => '▀',
        // Bottom brighter than top
        (2, 3) => '▄',
        (1, 3) => '▄',
        (0, 3) => '▄',
        // Similar medium-high brightness - use full block for cleaner look
        (2, 2) => '█',
        // Top medium, bottom lower
        (2, 1) => '▀',
        (2, 0) => '▀',
        // Bottom medium, top lower
        (1, 2) => '▄',
        (0, 2) => '▄',
        // Both medium-low - use medium shade only
        (1, 1) => '▒',
        // One low, one very low - use space instead of light shade to reduce noise
        (1, 0) | (0, 1) => ' ',
        // Both very dark - empty space
        _ => ' ',
    }
}

/// Calculate target size preserving aspect ratio
fn calculate_target_size(
    img_width: u32,
    img_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    // Terminal characters are roughly 1:2 width:height ratio (twice as tall as wide)
    // Since we use half-blocks (2 pixels per character vertically),
    // we need to compensate for the character aspect ratio
    
    // For each character width, we can fit ~2x more pixels horizontally
    // to maintain proper aspect ratio
    let char_width_factor = 2.0;
    
    // Calculate available pixel space considering character aspect ratio
    let available_pixel_width = (max_width as f64) * char_width_factor;
    let available_pixel_height = max_height as f64;
    
    // Calculate scale to fit image in available space
    let width_scale = available_pixel_width / (img_width as f64);
    let height_scale = available_pixel_height / (img_height as f64);
    
    // Use the smaller scale to ensure image fits
    let scale = width_scale.min(height_scale);
    
    // Calculate target dimensions in pixels
    let target_pixel_width = ((img_width as f64) * scale) as u32;
    let target_pixel_height = ((img_height as f64) * scale) as u32;
    
    (target_pixel_width.max(1), target_pixel_height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.jpeg")));
        assert!(is_image_file(Path::new("test.gif")));
        assert!(is_image_file(Path::new("test.bmp")));
        assert!(is_image_file(Path::new("test.webp")));

        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test.rs")));
        assert!(!is_image_file(Path::new("test.exe")));
    }

    #[test]
    fn test_calculate_target_size() {
        // Test landscape image
        let (w, h) = calculate_target_size(800, 600, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);
        assert!(w > 0 && h > 0);

        // Test portrait image
        let (w, h) = calculate_target_size(600, 800, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);
        assert!(w > 0 && h > 0);

        // Test square image
        let (w, h) = calculate_target_size(500, 500, 80, 40);
        assert!(w <= 80);
        assert!(h <= 40);
        assert!(w > 0 && h > 0);
    }

    #[test]
    fn test_detect_format_from_path() {
        assert_eq!(
            detect_format_from_path(Path::new("test.png")),
            Some("PNG".to_string())
        );
        assert_eq!(
            detect_format_from_path(Path::new("test.jpg")),
            Some("JPG".to_string())
        );
        assert_eq!(
            detect_format_from_path(Path::new("test.gif")),
            Some("GIF".to_string())
        );
        assert_eq!(detect_format_from_path(Path::new("test")), None);
    }
}
