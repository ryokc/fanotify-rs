use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;

use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags};

/// Configuration for the advanced monitor
struct MonitorConfig {
    /// Paths to monitor
    paths: Vec<PathBuf>,
    /// File extensions to filter (empty = all files)
    extensions: Vec<String>,
    /// Minimum file size to track (in bytes)
    min_file_size: u64,
    /// Whether to track directories
    track_directories: bool,
    /// Whether to track symbolic links
    track_symlinks: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            paths: vec![PathBuf::from("/tmp")],
            extensions: Vec::new(),
            min_file_size: 0,
            track_directories: true,
            track_symlinks: false,
        }
    }
}

/// Statistics for the monitor
#[derive(Default)]
struct MonitorStats {
    total_events: u64,
    access_events: u64,
    modify_events: u64,
    create_events: u64,
    delete_events: u64,
    move_events: u64,
    permission_events: u64,
    start_time: Instant,
    last_event_time: Option<Instant>,
}

impl MonitorStats {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            ..Default::default()
        }
    }
    
    fn record_event(&mut self, event: &fanotify_rs::Event) {
        self.total_events += 1;
        self.last_event_time = Some(Instant::now());
        
        if event.is_access() {
            self.access_events += 1;
        }
        if event.is_modify() {
            self.modify_events += 1;
        }
        if event.is_create() {
            self.create_events += 1;
        }
        if event.is_delete() {
            self.delete_events += 1;
        }
        if event.is_move() {
            self.move_events += 1;
        }
        if event.is_permission() {
            self.permission_events += 1;
        }
    }
    
    fn print_summary(&self) {
        let uptime = self.start_time.elapsed();
        let events_per_second = if uptime.as_secs() > 0 {
            self.total_events as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };
        
        println!("\n=== Monitor Statistics ===");
        println!("Uptime: {:.2} seconds", uptime.as_secs_f64());
        println!("Total events: {}", self.total_events);
        println!("Events per second: {:.2}", events_per_second);
        println!("\nEvent breakdown:");
        println!("  Access: {} ({:.1}%)", 
            self.access_events, 
            self.percentage(self.access_events));
        println!("  Modify: {} ({:.1}%)", 
            self.modify_events, 
            self.percentage(self.modify_events));
        println!("  Create: {} ({:.1}%)", 
            self.create_events, 
            self.percentage(self.create_events));
        println!("  Delete: {} ({:.1}%)", 
            self.delete_events, 
            self.percentage(self.delete_events));
        println!("  Move: {} ({:.1}%)", 
            self.move_events, 
            self.percentage(self.move_events));
        println!("  Permission: {} ({:.1}%)", 
            self.permission_events, 
            self.percentage(self.permission_events));
        println!("========================\n");
    }
    
    fn percentage(&self, count: u64) -> f64 {
        if self.total_events > 0 {
            (count as f64 / self.total_events as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Advanced file system monitor
struct AdvancedMonitor {
    fanotify: Fanotify,
    config: MonitorConfig,
    stats: MonitorStats,
    recent_files: HashMap<PathBuf, Instant>,
}

impl AdvancedMonitor {
    fn new(config: MonitorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let fanotify = Fanotify::with_flags(
            FanotifyFlags::NONBLOCK | FanotifyFlags::CLOEXEC
        )?;
        
        Ok(Self {
            fanotify,
            config,
            stats: MonitorStats::new(),
            recent_files: HashMap::new(),
        })
    }
    
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting advanced fanotify monitor...");
        println!("Configuration:");
        println!("  Paths: {:?}", self.config.paths);
        println!("  Extensions: {:?}", self.config.extensions);
        println!("  Min file size: {} bytes", self.config.min_file_size);
        println!("  Track directories: {}", self.config.track_directories);
        println!("  Track symlinks: {}", self.config.track_symlinks);
        println!();
        
        // Add watches for all configured paths
        for path in &self.config.paths {
            if path.exists() {
                self.fanotify.add_watch(path, MaskFlags::ALL_EVENTS)?;
                println!("Added watch for: {}", path.display());
            } else {
                eprintln!("Warning: Path does not exist: {}", path.display());
            }
        }
        
        println!("Monitoring started. Press Ctrl+C to stop.");
        
        let mut last_stats_time = Instant::now();
        
        // Main event loop
        loop {
            let events = self.fanotify.read_events()?;
            
            for event in events {
                if self.should_process_event(&event) {
                    self.process_event(&event);
                    self.stats.record_event(&event);
                }
            }
            
            // Print statistics every 30 seconds
            if last_stats_time.elapsed() >= Duration::from_secs(30) {
                self.stats.print_summary();
                last_stats_time = Instant::now();
            }
            
            thread::sleep(Duration::from_millis(100));
        }
    }
    
    fn should_process_event(&self, event: &fanotify_rs::Event) -> bool {
        // Skip if we don't want to track directories and this is a directory event
        if !self.config.track_directories && event.info.is_directory {
            return false;
        }
        
        // Check file extension filter
        if let Some(path) = &event.info.path {
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if !self.config.extensions.is_empty() && 
                       !self.config.extensions.contains(&ext_str.to_lowercase()) {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    fn process_event(&mut self, event: &fanotify_rs::Event) {
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        let event_type = event.event_type();
        
        if let Some(path) = &event.info.path {
            let path_str = path.display();
            
            // Update recent files tracking
            if event.is_create() || event.is_modify() {
                self.recent_files.insert(path.clone(), Instant::now());
            }
            
            // Print event details
            println!("[{}] {} - {} (PID: {})", 
                timestamp, event_type, path_str, event.info.pid);
            
            // Add additional context for certain events
            match event_type {
                "CREATE" => {
                    if let Some(filename) = event.info.filename() {
                        println!("  New file: {}", filename);
                    }
                },
                "DELETE" => {
                    if let Some(filename) = event.info.filename() {
                        println!("  Deleted file: {}", filename);
                    }
                },
                "MODIFY" => {
                    if let Some(filename) = event.info.filename() {
                        println!("  Modified file: {}", filename);
                    }
                },
                "MOVE" => {
                    println!("  Move operation detected");
                },
                _ => {}
            }
        }
        
        // Clean up old entries from recent_files
        let cutoff = Instant::now() - Duration::from_secs(300); // 5 minutes
        self.recent_files.retain(|_, time| *time > cutoff);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let mut config = MonitorConfig::default();
    
    // Simple argument parsing
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--paths" => {
                if i + 1 < args.len() {
                    config.paths = args[i + 1]
                        .split(',')
                        .map(|s| PathBuf::from(s.trim()))
                        .collect();
                }
            },
            "--extensions" => {
                if i + 1 < args.len() {
                    config.extensions = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .collect();
                }
            },
            "--min-size" => {
                if i + 1 < args.len() {
                    if let Ok(size) = args[i + 1].parse::<u64>() {
                        config.min_file_size = size;
                    }
                }
            },
            "--no-dirs" => {
                config.track_directories = false;
            },
            "--symlinks" => {
                config.track_symlinks = true;
            },
            _ => {}
        }
    }
    
    // Create and start the monitor
    let mut monitor = AdvancedMonitor::new(config)?;
    monitor.start()
} 