#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! sled = "0.34"
//! bincode = "1.3"
//! ```

use std::collections::HashMap;

fn main() {
    let db_path = "data/tinymush";
    println!("Opening Sled database at: {}", db_path);
    
    let db = match sled::open(db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            return;
        }
    };
    
    println!("\nScanning for NPC records...\n");
    
    for npc_id in &["mayor_thompson", "city_clerk", "gate_guard", "market_vendor", "museum_curator"] {
        let key = format!("npcs:{}", npc_id);
        match db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                println!("Found NPC: {}", npc_id);
                println!("  Raw data length: {} bytes", data.len());
                println!("  First 50 bytes (hex): {:02x?}", &data[0..data.len().min(50)]);
                
                // Try to decode as bincode
                match bincode::deserialize::<HashMap<String, serde_json::Value>>(&data) {
                    Ok(map) => {
                        println!("  Successfully decoded as HashMap");
                        for (k, v) in map.iter().take(5) {
                            println!("    {}: {:?}", k, v);
                        }
                    }
                    Err(e) => {
                        println!("  Failed to decode: {}", e);
                    }
                }
                println!();
            }
            Ok(None) => {
                println!("NPC not found: {}\n", npc_id);
            }
            Err(e) => {
                println!("Error reading NPC {}: {}\n", npc_id, e);
            }
        }
    }
}
