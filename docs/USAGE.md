# fanotify-rs Usage Guide

This guide provides detailed information on how to use the fanotify-rs library effectively.

## Table of Contents

1. [Basic Concepts](#basic-concepts)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Synchronous Usage](#synchronous-usage)
5. [Asynchronous Usage](#asynchronous-usage)
6. [Event Handling](#event-handling)
7. [Permission Events](#permission-events)
8. [Error Handling](#error-handling)
9. [Best Practices](#best-practices)
10. [Performance Considerations](#performance-considerations)
11. [Troubleshooting](#troubleshooting)

## Basic Concepts

### What is fanotify?

Fanotify is a Linux kernel feature that provides filesystem event notification. It allows applications to:

- Monitor filesystem events (file access, modification, creation, deletion, etc.)
- Control access to files and directories
- Receive real-time notifications about filesystem changes

### Key Components

- **Fanotify**: The main synchronous wrapper
- **AsyncFanotify**: The asynchronous wrapper for use with tokio
- **Event**: Represents a filesystem event
- **MaskFlags**: Defines which events to monitor
- **FanotifyFlags**: Configuration flags for the fanotify instance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fanotify-rs = "0.1.0"
```

## Quick Start

### Basic File Monitoring

```rust
use fanotify_rs::{Fanotify, MaskFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a fanotify instance
    let mut fanotify = Fanotify::new()?;
    
    // Monitor a directory for all events
    fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS)?;
    
    // Read and process events
    for event in fanotify.events() {
        match event {
            Ok(event) => {
                println!("Event: {:?}", event);
                if let Some(path) = &event.info.path {
                    println!("  Path: {}", path.display());
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

## Synchronous Usage

### Creating a Fanotify Instance

```rust
use fanotify_rs::{Fanotify, FanotifyFlags};

// With default flags
let mut fanotify = Fanotify::new()?;

// With custom flags
let mut fanotify = Fanotify::with_flags(
    FanotifyFlags::NONBLOCK | FanotifyFlags::CLOEXEC
)?;
```

### Adding Watches

```rust
use fanotify_rs::MaskFlags;

// Monitor all events
fanotify.add_watch("/path/to/monitor", MaskFlags::ALL_EVENTS)?;

// Monitor specific events
let mask = MaskFlags::CREATE | MaskFlags::DELETE | MaskFlags::MODIFY;
fanotify.add_watch("/path/to/monitor", mask)?;

// Monitor access events only
fanotify.add_watch("/path/to/monitor", MaskFlags::ALL_ACCESS_EVENTS)?;
```

### Reading Events

```rust
// Read a single event
match fanotify.read_event()? {
    Some(event) => println!("Event: {:?}", event),
    None => println!("No events available"),
}

// Read all available events
let events = fanotify.read_events()?;
for event in events {
    println!("Event: {:?}", event);
}

// Use iterator
for event in fanotify.events() {
    match event {
        Ok(event) => println!("Event: {:?}", event),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Asynchronous Usage

### Basic Async Example

```rust
use fanotify_rs::{AsyncFanotify, MaskFlags};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fanotify = AsyncFanotify::new()?;
    
    fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS).await?;
    
    while let Some(event) = fanotify.next_event().await? {
        println!("Async event: {:?}", event);
    }
    
    Ok(())
}
```

### Waiting for Events

```rust
// Wait for the next event (blocks until available)
let event = fanotify.wait_for_event().await?;
println!("Received event: {:?}", event);
```

## Event Handling

### Event Types

```rust
let event = fanotify.read_event()?.unwrap();

// Check event type
if event.is_access() {
    println!("File accessed");
}
if event.is_modify() {
    println!("File modified");
}
if event.is_create() {
    println!("File created");
}
if event.is_delete() {
    println!("File deleted");
}
if event.is_move() {
    println!("File moved");
}
if event.is_permission() {
    println!("Permission event");
}
```

### Event Information

```rust
let event = fanotify.read_event()?.unwrap();

// Get event metadata
let info = &event.info;
println!("Process ID: {}", info.pid);
println!("Is directory: {}", info.is_directory);

// Get file path
if let Some(path) = &info.path {
    println!("File path: {}", path.display());
}

// Get filename
if let Some(filename) = info.filename() {
    println!("Filename: {}", filename);
}

// Get event description
println!("Event: {}", event.description());
```

## Permission Events

### Handling Permission Events

```rust
for event in fanotify.events() {
    let event = event?;
    
    if event.is_permission() {
        // Decide whether to allow or deny access
        if should_allow_access(&event) {
            fanotify.allow(&event)?;
            println!("Allowed access to: {:?}", event.info.path);
        } else {
            fanotify.deny(&event)?;
            println!("Denied access to: {:?}", event.info.path);
        }
    }
}

fn should_allow_access(event: &fanotify_rs::Event) -> bool {
    // Implement your access control logic here
    // For example, check process ID, user, file path, etc.
    true
}
```

### Permission Event Types

```rust
// Monitor for permission events
let mask = MaskFlags::OPEN_PERM | MaskFlags::ACCESS_PERM;
fanotify.add_watch("/sensitive/directory", mask)?;
```

## Error Handling

### Common Errors

```rust
use fanotify_rs::{FanotifyError, Result};

fn handle_fanotify_errors() -> Result<()> {
    let mut fanotify = Fanotify::new()?;
    
    match fanotify.add_watch("/nonexistent/path", MaskFlags::ALL_EVENTS) {
        Ok(_) => println!("Watch added successfully"),
        Err(FanotifyError::InvalidPath { path }) => {
            eprintln!("Invalid path: {}", path);
        }
        Err(FanotifyError::PermissionDenied { message }) => {
            eprintln!("Permission denied: {}", message);
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

### Error Recovery

```rust
fn robust_fanotify_usage() -> Result<()> {
    let mut fanotify = Fanotify::new()?;
    
    // Try to add watch with error handling
    for path in ["/tmp", "/var/log", "/home"] {
        match fanotify.add_watch(path, MaskFlags::ALL_EVENTS) {
            Ok(_) => println!("Added watch for {}", path),
            Err(FanotifyError::PermissionDenied { message }) => {
                eprintln!("Cannot watch {}: {}", path, message);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(())
}
```

## Best Practices

### 1. Resource Management

```rust
// Always use proper error handling
let mut fanotify = Fanotify::new()?;

// Clean up watches when done
fanotify.remove_watch("/path/to/monitor")?;

// The fanotify instance will be automatically closed when dropped
```

### 2. Event Processing

```rust
// Process events efficiently
for event in fanotify.events() {
    match event {
        Ok(event) => {
            // Handle event quickly to avoid blocking
            process_event_quickly(&event);
        }
        Err(e) => {
            // Log errors but continue processing
            eprintln!("Event error: {}", e);
        }
    }
}
```

### 3. Buffer Management

```rust
// Adjust buffer size for your use case
let mut fanotify = Fanotify::new()?;
fanotify.set_buffer_size(8192); // 8KB buffer
```

### 4. Multiple Watches

```rust
// Monitor multiple directories efficiently
let paths = ["/tmp", "/var/log", "/home/user"];
let mask = MaskFlags::ALL_EVENTS;

for path in &paths {
    fanotify.add_watch(path, mask)?;
}
```

## Performance Considerations

### 1. Event Filtering

```rust
// Only monitor events you need
let mask = MaskFlags::CREATE | MaskFlags::DELETE; // Only creation/deletion
fanotify.add_watch("/path", mask)?;
```

### 2. Non-blocking Mode

```rust
// Use non-blocking mode for responsive applications
let mut fanotify = Fanotify::with_flags(FanotifyFlags::NONBLOCK)?;
```

### 3. Batch Processing

```rust
// Process events in batches
let events = fanotify.read_events()?;
for event in events {
    // Process event
}
```

### 4. Async Processing

```rust
// Use async for better performance in I/O-bound applications
let mut fanotify = AsyncFanotify::new()?;
while let Some(event) = fanotify.next_event().await? {
    // Process event asynchronously
}
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   # Run with appropriate permissions
   sudo cargo run --example basic_monitor
   ```

2. **Kernel Not Supported**
   ```bash
   # Check kernel version
   uname -r
   # Should be 2.6.36 or later
   ```

3. **No Events Received**
   ```rust
   // Ensure you're monitoring the right events
   let mask = MaskFlags::ALL_EVENTS; // Monitor all events
   fanotify.add_watch("/path", mask)?;
   ```

4. **High CPU Usage**
   ```rust
   // Add sleep to prevent busy waiting
   thread::sleep(Duration::from_millis(100));
   ```

### Debugging

```rust
// Enable debug logging
use std::env;
env::set_var("RUST_LOG", "debug");

// Check fanotify capabilities
if let Err(FanotifyError::NotSupported) = Fanotify::new() {
    eprintln!("Fanotify not supported by kernel");
    return;
}
```

### Performance Monitoring

```rust
// Monitor event processing performance
let start = std::time::Instant::now();
let events = fanotify.read_events()?;
let duration = start.elapsed();
println!("Processed {} events in {:?}", events.len(), duration);
```

## Examples

See the `examples/` directory for complete working examples:

- `basic_monitor.rs`: Simple file monitoring
- `async_monitor.rs`: Asynchronous monitoring
- `permission_monitor.rs`: Permission-based access control
- `advanced_monitor.rs`: Advanced monitoring with filtering and statistics

Run examples with:

```bash
cargo run --example basic_monitor /path/to/monitor
cargo run --example async_monitor /path/to/monitor
cargo run --example permission_monitor /path/to/monitor
cargo run --example advanced_monitor -- --paths /tmp,/var/log
``` 