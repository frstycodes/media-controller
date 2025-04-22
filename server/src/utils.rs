use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose;
use image;

/// Encodes raw image bytes as a base64 data URL for use in HTML/CSS
pub fn encode_image_to_base64(bytes: &[u8]) -> String {
    let encoder = general_purpose::STANDARD;
    format!("data:image/jpeg;base64,{}", encoder.encode(bytes))
}

/// Extracts a dominant hue value (0-360) from an image to use for UI theming
///
/// Returns the hue value in the standard 0-360 degree range that can be used
/// for setting accent colors in the UI that match the album art.
///
/// # Arguments
///
/// * `image_bytes` - Raw bytes of the image (typically album art)
///
/// # Returns
///
/// * `Result<u16>` - Hue value between 0-360, or an error if image processing fails
pub fn extract_accent_color_hue(image_bytes: &[u8]) -> Result<u16> {
    // Load the image from bytes
    let img = image::load_from_memory(image_bytes)?;

    // Resize to something small for faster processing
    let small = img.resize(32, 32, image::imageops::FilterType::Nearest);

    // Convert to RGB for easier color analysis
    let rgb_img = small.to_rgb8();

    // Average RGB values across the image to find dominant color
    let mut r_sum: u64 = 0;
    let mut g_sum: u64 = 0;
    let mut b_sum: u64 = 0;
    let mut pixel_count: u64 = 0;

    for pixel in rgb_img.pixels() {
        // Skip very dark/black pixels as they don't contribute to accent color
        if pixel[0] < 30 && pixel[1] < 30 && pixel[2] < 30 {
            continue;
        }

        r_sum += pixel[0] as u64;
        g_sum += pixel[1] as u64;
        b_sum += pixel[2] as u64;
        pixel_count += 1;
    }

    // If we found no valid pixels, use a default hue
    if pixel_count == 0 {
        return Ok(141); // Default cyan-ish hue
    }

    // Calculate average RGB
    let avg_r = (r_sum / pixel_count) as f32;
    let avg_g = (g_sum / pixel_count) as f32;
    let avg_b = (b_sum / pixel_count) as f32;

    // Convert RGB to HSV - we only care about the Hue (H) component
    let (h, _s, _v) = rgb_to_hsv(avg_r, avg_g, avg_b);

    // Return the hue directly in 0-360 range
    Ok(h.round() as u16)
}

/// Convert RGB color values to HSV (Hue, Saturation, Value)
///
/// # Arguments
///
/// * `r`, `g`, `b` - RGB values in range 0-255 as f32
///
/// # Returns
///
/// * `(h, s, v)` - Hue (0-360), Saturation (0-1), Value (0-1)
fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Calculate hue
    let h = if delta == 0.0 {
        0.0 // No hue for grayscale colors
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    // Ensure positive hue
    let h = if h < 0.0 { h + 360.0 } else { h };

    // Calculate saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Value is just the max
    let v = max;

    (h, s, v)
}
