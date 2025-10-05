//! Migration script to add message_id and crc16 fields to existing messages
//!
//! Usage: cargo run --bin migrate_messages -- /path/to/data

use chrono::{DateTime, Utc};
use crc::{Crc, CRC_16_IBM_SDLC};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CRC_CALCULATOR: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    pub id: String,
    pub topic: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub replies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crc16: Option<u16>,
}

fn generate_message_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 2] = [rng.gen(), rng.gen()];
    
    let timestamp_hex = format!("{:08x}", now);
    let random_hex = format!("{:02x}{:02x}", random_bytes[0], random_bytes[1]);
    
    format!("{}{}", timestamp_hex, random_hex)
}

fn calculate_message_crc(topic: &str, author: &str, content: &str, timestamp: &DateTime<Utc>) -> u16 {
    let mut digest = CRC_CALCULATOR.digest();
    digest.update(topic.as_bytes());
    digest.update(author.as_bytes());
    digest.update(content.as_bytes());
    digest.update(timestamp.to_rfc3339().as_bytes());
    digest.finalize()
}

fn find_message_files(data_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let messages_dir = data_dir.join("messages");
    let mut files = Vec::new();
    
    if !messages_dir.exists() {
        eprintln!("Warning: messages directory does not exist: {:?}", messages_dir);
        return Ok(files);
    }
    
    for entry in std::fs::read_dir(&messages_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // This is a topic directory
            for msg_entry in std::fs::read_dir(&path)? {
                let msg_entry = msg_entry?;
                let msg_path = msg_entry.path();
                
                if msg_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    files.push(msg_path);
                }
            }
        }
    }
    
    Ok(files)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <data_directory>", args[0]);
        eprintln!("Example: {} /opt/meshbbs/data", args[0]);
        std::process::exit(1);
    }
    
    let data_dir = Path::new(&args[1]);
    
    if !data_dir.exists() {
        eprintln!("Error: Data directory does not exist: {:?}", data_dir);
        std::process::exit(1);
    }
    
    println!("Migrating messages in: {:?}", data_dir);
    println!();
    
    let message_files = find_message_files(data_dir)?;
    
    if message_files.is_empty() {
        println!("No message files found. Nothing to migrate.");
        return Ok(());
    }
    
    println!("Found {} message files", message_files.len());
    
    let mut migrated = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let mut used_ids = HashSet::new();
    
    for msg_path in &message_files {
        let content = match std::fs::read_to_string(msg_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", msg_path, e);
                errors += 1;
                continue;
            }
        };
        
        let mut message: Message = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error parsing {:?}: {}", msg_path, e);
                errors += 1;
                continue;
            }
        };
        
        // Check if already migrated
        if message.message_id.is_some() && message.crc16.is_some() {
            skipped += 1;
            continue;
        }
        
        // Generate unique message_id
        let mut msg_id = generate_message_id();
        while used_ids.contains(&msg_id) {
            // Sleep a tiny bit to ensure different timestamp
            std::thread::sleep(std::time::Duration::from_millis(1));
            msg_id = generate_message_id();
        }
        used_ids.insert(msg_id.clone());
        
        // Calculate CRC
        let crc = calculate_message_crc(
            &message.topic,
            &message.author,
            &message.content,
            &message.timestamp,
        );
        
        // Update message
        message.message_id = Some(msg_id.clone());
        message.crc16 = Some(crc);
        
        // Write back to file
        let json = match serde_json::to_string_pretty(&message) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Error serializing {:?}: {}", msg_path, e);
                errors += 1;
                continue;
            }
        };
        
        if let Err(e) = std::fs::write(msg_path, json) {
            eprintln!("Error writing {:?}: {}", msg_path, e);
            errors += 1;
            continue;
        }
        
        migrated += 1;
        
        if migrated % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    
    println!();
    println!();
    println!("Migration complete!");
    println!("  Migrated: {}", migrated);
    println!("  Skipped (already migrated): {}", skipped);
    println!("  Errors: {}", errors);
    println!("  Total: {}", message_files.len());
    
    if errors > 0 {
        println!();
        println!("⚠️  Some messages encountered errors during migration.");
        println!("   Please review the error messages above.");
    } else if migrated > 0 {
        println!();
        println!("✓ All messages successfully migrated!");
    }
    
    Ok(())
}
