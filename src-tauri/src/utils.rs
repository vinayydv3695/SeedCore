//! Utility functions for SeedCore

use rand::Rng;

/// Generate a random peer ID
///
/// Format: -SC0100-<12 random chars>
/// SC = SeedCore, 0100 = version 0.1.0
pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];

    // Prefix: -SC0100-
    peer_id[0] = b'-';
    peer_id[1] = b'S';
    peer_id[2] = b'C';
    peer_id[3] = b'0';
    peer_id[4] = b'1';
    peer_id[5] = b'0';
    peer_id[6] = b'0';
    peer_id[7] = b'-';

    // Random suffix (12 characters)
    let mut rng = rand::thread_rng();
    for byte in &mut peer_id[8..20] {
        // Use printable ASCII characters (33-126)
        *byte = rng.gen_range(33..=126);
    }

    peer_id
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format speed in bytes/sec
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Format duration in seconds as human-readable time
pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Calculate ETA (estimated time of arrival)
pub fn calculate_eta(remaining_bytes: u64, download_speed: u64) -> Option<u64> {
    if download_speed == 0 {
        return None;
    }

    Some(remaining_bytes / download_speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_peer_id() {
        let peer_id = generate_peer_id();

        // Check length
        assert_eq!(peer_id.len(), 20);

        // Check prefix
        assert_eq!(&peer_id[0..8], b"-SC0100-");

        // Check that random part is printable ASCII
        for &byte in &peer_id[8..20] {
            assert!(
                (33..=126).contains(&byte),
                "Byte {} is not printable ASCII",
                byte
            );
        }

        // Generate two peer IDs and ensure they're different (extremely unlikely to be same)
        let peer_id2 = generate_peer_id();
        assert_ne!(peer_id, peer_id2);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(100), "100 B");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1536), "1.50 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GiB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GiB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(1024), "1.00 KiB/s");
        assert_eq!(format_speed(1024 * 1024), "1.00 MiB/s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }

    #[test]
    fn test_calculate_eta() {
        assert_eq!(calculate_eta(1024 * 1024, 1024), Some(1024));
        assert_eq!(calculate_eta(1000, 0), None);
    }
}
