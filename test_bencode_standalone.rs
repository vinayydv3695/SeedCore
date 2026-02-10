// Quick standalone bencode test
#[path = "src-tauri/src/bencode.rs"]
mod bencode;

#[path = "src-tauri/src/error.rs"]
mod error;

use bencode::BencodeValue;

fn main() {
    // Test parsing a dictionary with "piece length" key (with space!)
    let data = b"d12:piece lengthi16384ee";

    println!("Test 1 - Simple dict with 'piece length' key:");
    println!("Input: {:?}", std::str::from_utf8(data).unwrap());

    match BencodeValue::parse(data) {
        Ok(value) => println!("✓ SUCCESS: {:#?}\n", value),
        Err(e) => println!("✗ ERROR: {}\n", e),
    }

    // Test the full torrent data
    let full_data = b"d8:announce15:http://tracker4:infod6:lengthi1234e4:name9:test.file12:piece lengthi16384e6:pieces20:12345678901234567890ee";

    println!("Test 2 - Full torrent structure:");
    match BencodeValue::parse(full_data) {
        Ok(value) => println!("✓ SUCCESS: {:#?}", value),
        Err(e) => println!("✗ ERROR: {}", e),
    }
}
