use std::path::Path;
use std::time::Duration;
use std::thread;

use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting fanotify monitor...");
    
    // Create a new fanotify instance with non-blocking flag
    let mut fanotify = Fanotify::with_flags(
        FanotifyFlags::NONBLOCK | FanotifyFlags::CLOEXEC
    )?;
    
    // Get the directory to monitor from command line args or use current directory
    let monitor_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());
    
    println!("Monitoring directory: {}", monitor_path);
    
    // Add a watch for the specified directory
    // Monitor for all events except permission events
    let mask = MaskFlags::ALL_ACCESS_EVENTS | MaskFlags::ALL_MODIFY_EVENTS;
    fanotify.add_watch(&monitor_path, mask)?;
    
    println!("Watch added successfully. Press Ctrl+C to stop.");
    
    // Event monitoring loop
    let mut event_count = 0;
    loop {
        // Read all available events
        let events = fanotify.read_events()?;
        
        for event in events {
            event_count += 1;
            println!("Event #{}: {}", event_count, event.description());
            
            if let Some(path) = &event.info.path {
                println!("  Path: {}", path.display());
            }
            
            if let Some(filename) = event.info.filename() {
                println!("  File: {}", filename);
            }
            
            println!("  Process ID: {}", event.info.pid);
            println!("  Event type: {}", event.event_type());
            println!("  Is directory: {}", event.info.is_directory);
            println!();
        }
        
        // Small delay to prevent busy waiting
        thread::sleep(Duration::from_millis(100));
    }
} 