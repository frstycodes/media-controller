use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose;
use image;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// Define common constants for server use
pub const FRONTEND_PORT: u16 = 5173;
pub const SOCKETIO_PORT: u16 = 5174;
pub const ADDR: [u8; 4] = [0, 0, 0, 0];

// Image processing utilities
pub fn encode_image_to_base64(bytes: &[u8]) -> String {
    let encoder = general_purpose::STANDARD;
    format!("data:image/jpeg;base64,{}", encoder.encode(bytes))
}

/// Extracts a dominant hue value (0-360)
///
/// # Arguments
/// * `image_bytes` - Raw bytes of the image
///
/// # Returns
/// * `Result<u16>` - Hue value between 0-360
pub fn extract_accent_color_hue(image_bytes: &[u8]) -> Result<u16> {
    let img = image::load_from_memory(image_bytes)?;
    let small = img.resize(32, 32, image::imageops::FilterType::Gaussian);

    // Convert to RGB for easier color analysis
    let rgb_img = small.to_rgb8();

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
        return Ok(148); // Defaults to emerald green
    }

    // Calculate average RGB
    let r = (r_sum / pixel_count) as f32;
    let g = (g_sum / pixel_count) as f32;
    let b = (b_sum / pixel_count) as f32;

    let (h, ..) = rgb_to_hsv(r, g, b);

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

// Server and network utilities

/// Try to bind to specified port, fall back to random port if unavailable
///
/// # Arguments
/// * `preferred_port` - The port to try binding to first
///
/// # Returns
/// * `Result<(TcpListener, u16)>` - The listener and the actual port number used
pub async fn try_bind(preferred_port: u16) -> Result<(TcpListener, u16)> {
    // First try the preferred port
    let preferred_addr = SocketAddr::from((ADDR, preferred_port));
    match TcpListener::bind(preferred_addr).await {
        Ok(listener) => Ok((listener, preferred_port)),
        Err(_) => {
            // If preferred port is unavailable, bind to port 0 (OS will assign random available port)
            let random_addr = SocketAddr::from((ADDR, 0));
            let listener = TcpListener::bind(random_addr).await?;
            let actual_port = listener.local_addr()?.port();
            Ok((listener, actual_port))
        }
    }
}

/// Get local network IP addresses
///
/// # Returns
/// * `Vec<String>` - List of network IP addresses
pub fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();

    // Primary method: Connect to a public address and see what interface is used
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        // This doesn't actually send any data, just gives us the interface that would be used
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let std::net::SocketAddr::V4(addr) = addr {
                    let ip = addr.ip();
                    if !ip.is_loopback() {
                        ips.push(ip.to_string());
                    }
                }
            }
        }
    }

    ips
}

/// Format text with green color for terminal
///
/// # Arguments
/// * `format` - The format string
/// * `args` - Format arguments
///
/// # Returns
/// * `String` - Green colored string
#[macro_export]
macro_rules! green_print {
    ($($arg:tt)*) => {
        format!("\x1b[32m{}\x1b[0m", format!($($arg)*))
    };
}

/// Print URLs in a formatted, clickable way
///
/// # Arguments
/// * `service_name` - Name of the service (e.g., "Frontend", "SocketIO")
/// * `port` - Port number the service is running on
pub fn print_urls(service_name: &str, port: u16) {
    println!("\n  {service_name} server running at:\n");

    // Local URL (clickable in most terminals)
    println!("  > Local:    \x1b[32mhttp://localhost:{port}/\x1b[0m");

    // Network URLs
    let network_ips = get_local_ips();
    if !network_ips.is_empty() {
        for (i, ip) in network_ips.iter().enumerate() {
            if i == 0 {
                println!("  > Network:  \x1b[32mhttp://{ip}:{port}/\x1b[0m");
            } else {
                println!("              \x1b[32mhttp://{ip}:{port}/\x1b[0m");
            }
        }
    } else {
        println!("  > Network:  \x1b[33munavailable\x1b[0m");
    }

    // Health check URL
    println!("\n  > Health:   \x1b[32mhttp://localhost:{port}/health\x1b[0m\n");
}
