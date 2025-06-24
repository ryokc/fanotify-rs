use std::path::Path;
use std::time::Duration;

use fanotify_rs::{AsyncFanotify, FanotifyFlags, MaskFlags};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting async fanotify monitor...");
    
    // Create a new async fanotify instance
    let mut fanotify = AsyncFanotify::with_flags(
        FanotifyFlags::NONBLOCK | FanotifyFlags::CLOEXEC
    )?;
    
    // Get the directory to monitor from command line args or use current directory
    let monitor_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());
    
    println!("Monitoring directory: {}", monitor_path);
    
    // Add a watch for the specified directory
    let mask = MaskFlags::ALL_ACCESS_EVENTS | MaskFlags::ALL_MODIFY_EVENTS;
    fanotify.add_watch(&monitor_path, mask).await?;
    
    println!("Watch added successfully. Press Ctrl+C to stop.");
    
    // Async event monitoring loop
    let mut event_count = 0;
    loop {
        // Try to get the next event
        match fanotify.next_event().await? {
            Some(event) => {
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
            None => {
                // No events available, wait a bit before trying again
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
} 