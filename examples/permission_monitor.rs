use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use std::thread;

use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags, EventFlags};

/// Simple access control based on process ID
struct AccessController {
    allowed_pids: HashMap<u32, String>,
    denied_paths: Vec<String>,
}

impl AccessController {
    fn new() -> Self {
        let mut controller = Self {
            allowed_pids: HashMap::new(),
            denied_paths: Vec::new(),
        };
        
        // Add some example allowed processes
        controller.allowed_pids.insert(1, "systemd".to_string());
        controller.allowed_pids.insert(1000, "user_process".to_string());
        
        // Add some denied paths
        controller.denied_paths.push("/etc/shadow".to_string());
        controller.denied_paths.push("/etc/passwd".to_string());
        
        controller
    }
    
    fn should_allow(&self, event: &fanotify_rs::Event) -> bool {
        let pid = event.info.pid;
        
        // Check if process is explicitly allowed
        if self.allowed_pids.contains_key(&pid) {
            return true;
        }
        
        // Check if path is explicitly denied
        if let Some(path) = &event.info.path {
            let path_str = path.to_string_lossy();
            for denied_path in &self.denied_paths {
                if path_str.contains(denied_path) {
                    return false;
                }
            }
        }
        
        // Default to allowing access
        true
    }
    
    fn get_process_name(&self, pid: u32) -> String {
        self.allowed_pids.get(&pid)
            .cloned()
            .unwrap_or_else(|| format!("unknown_pid_{}", pid))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting fanotify permission monitor...");
    println!("This example demonstrates permission-based access control.");
    println!("Press Ctrl+C to stop.");
    
    // Create access controller
    let controller = AccessController::new();
    
    // Create fanotify instance
    let mut fanotify = Fanotify::with_flags(
        FanotifyFlags::CLOEXEC
    )?;
    
    // Get the directory to monitor from command line args or use root
    let monitor_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/".to_string());
    
    println!("Monitoring directory: {}", monitor_path);
    
    // Monitor for permission events
    let mask = MaskFlags::OPEN_PERM | MaskFlags::ACCESS_PERM;
    fanotify.add_watch(&monitor_path, mask)?;
    
    println!("Permission monitoring active. Access attempts will be logged and controlled.");
    
    let mut event_count = 0;
    let mut allowed_count = 0;
    let mut denied_count = 0;
    
    // Event monitoring loop
    loop {
        let events = fanotify.read_events()?;
        
        for event in events {
            event_count += 1;
            
            if event.is_permission() {
                let process_name = controller.get_process_name(event.info.pid);
                let path_str = event.info.path_str().unwrap_or("unknown");
                
                println!("Permission request #{}:", event_count);
                println!("  Process: {} (PID: {})", process_name, event.info.pid);
                println!("  Path: {}", path_str);
                println!("  Event type: {}", event.event_type());
                
                // Decide whether to allow or deny
                if controller.should_allow(&event) {
                    fanotify.allow(&event)?;
                    allowed_count += 1;
                    println!("  Decision: ALLOWED");
                } else {
                    fanotify.deny(&event)?;
                    denied_count += 1;
                    println!("  Decision: DENIED");
                }
                println!();
            }
        }
        
        // Print statistics every 10 seconds
        if event_count % 100 == 0 && event_count > 0 {
            println!("Statistics:");
            println!("  Total events: {}", event_count);
            println!("  Allowed: {}", allowed_count);
            println!("  Denied: {}", denied_count);
            println!("  Allow rate: {:.1}%", 
                (allowed_count as f64 / event_count as f64) * 100.0);
            println!();
        }
        
        thread::sleep(Duration::from_millis(100));
    }
} 