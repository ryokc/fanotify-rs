# fanotify-rs

An idiomatic Rust wrapper for Linux's fanotify API, providing safe and ergonomic filesystem event monitoring.

## Features

- **Safe abstractions**: All unsafe operations are wrapped in safe Rust code
- **Error handling**: Comprehensive error types with detailed information
- **Async support**: Both synchronous and asynchronous event monitoring
- **Type safety**: Strongly typed flags and event types
- **Documentation**: Extensive documentation with examples

## Requirements

- Linux kernel 2.6.36 or later (for fanotify support)
- Rust 1.70 or later
- Root privileges (for some operations)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fanotify-rs = "0.1.0"
```

## Quick Start

### Basic Usage

```rust
use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new fanotify instance
    let mut fanotify = Fanotify::new()?;
    
    // Monitor a directory for all events
    fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS)?;
    
    // Read events
    for event in fanotify.events() {
        match event {
            Ok(event) => println!("Event: {:?}", event),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

### Async Usage

```rust
use fanotify_rs::{AsyncFanotify, FanotifyFlags, MaskFlags};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new async fanotify instance
    let mut fanotify = AsyncFanotify::new()?;
    
    // Monitor a directory
    fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS).await?;
    
    // Read events asynchronously
    while let Some(event) = fanotify.next_event().await? {
        println!("Async event: {:?}", event);
    }
    
    Ok(())
}
```

## Examples

### Monitor Directory for File Changes

```rust
use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags};
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fanotify = Fanotify::with_flags(
        FanotifyFlags::NONBLOCK | FanotifyFlags::CLOEXEC
    )?;
    
    // Monitor for file modifications
    let mask = MaskFlags::MODIFY | MaskFlags::CREATE | MaskFlags::DELETE;
    fanotify.add_watch("/path/to/monitor", mask)?;
    
    println!("Monitoring for file changes...");
    
    loop {
        let events = fanotify.read_events()?;
        
        for event in events {
            match event.event_type() {
                "MODIFY" => println!("File modified: {:?}", event.info.path),
                "CREATE" => println!("File created: {:?}", event.info.path),
                "DELETE" => println!("File deleted: {:?}", event.info.path),
                _ => println!("Other event: {:?}", event),
            }
        }
        
        thread::sleep(Duration::from_millis(100));
    }
}
```

### Permission-Based Access Control

```rust
use fanotify_rs::{Fanotify, FanotifyFlags, MaskFlags, EventFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fanotify = Fanotify::new()?;
    
    // Monitor for permission events
    fanotify.add_watch("/sensitive/directory", MaskFlags::OPEN_PERM)?;
    
    for event in fanotify.events() {
        let event = event?;
        
        if event.is_permission() {
            // Check if the process should be allowed
            if should_allow_access(&event) {
                fanotify.allow(&event)?;
                println!("Allowed access to: {:?}", event.info.path);
            } else {
                fanotify.deny(&event)?;
                println!("Denied access to: {:?}", event.info.path);
            }
        }
    }
    
    Ok(())
}

fn should_allow_access(event: &fanotify_rs::Event) -> bool {
    // Implement your access control logic here
    // For example, check the process ID, user, etc.
    true
}
```

## API Reference

### Fanotify

The main synchronous fanotify wrapper.

#### Methods

- `new() -> Result<Self>`: Create a new fanotify instance with default flags
- `with_flags(flags: FanotifyFlags) -> Result<Self>`: Create with custom flags
- `add_watch<P: AsRef<Path>>(path: P, mask: MaskFlags) -> Result<()>`: Add a watch
- `remove_watch<P: AsRef<Path>>(path: P) -> Result<()>`: Remove a watch
- `read_event() -> Result<Option<Event>>`: Read a single event
- `read_events() -> Result<Vec<Event>>`: Read all available events
- `events() -> EventIterator`: Get an iterator over events
- `respond(event: &Event, response: EventFlags) -> Result<()>`: Respond to permission events
- `allow(event: &Event) -> Result<()>`: Allow a permission event
- `deny(event: &Event) -> Result<()>`: Deny a permission event

### AsyncFanotify

The asynchronous fanotify wrapper.

#### Methods

- `new() -> Result<Self>`: Create a new async fanotify instance
- `with_flags(flags: FanotifyFlags) -> Result<Self>`: Create with custom flags
- `add_watch<P: AsRef<Path>>(path: P, mask: MaskFlags) -> Result<()>`: Add a watch (async)
- `remove_watch<P: AsRef<Path>>(path: P) -> Result<()>`: Remove a watch (async)
- `read_event() -> Result<Option<Event>>`: Read a single event (async)
- `next_event() -> Result<Option<Event>>`: Get the next event (async)
- `wait_for_event() -> Result<Event>`: Wait for the next event (async)
- `respond(event: &Event, response: EventFlags) -> Result<()>`: Respond to permission events (async)

### Flags

#### FanotifyFlags

Flags for fanotify initialization:

- `CLOEXEC`: Close-on-exec flag
- `NONBLOCK`: Non-blocking flag
- `RDONLY`: Read-only flag
- `WRONLY`: Write-only flag
- `RDWR`: Read-write flag

#### MaskFlags

Event mask flags:

- `ACCESS`: File access
- `MODIFY`: File modification
- `ATTRIB`: Attribute change
- `CLOSE_WRITE`: File closed for writing
- `CLOSE_NOWRITE`: File closed for reading
- `OPEN`: File opened
- `MOVED_FROM`: File moved from
- `MOVED_TO`: File moved to
- `CREATE`: File created
- `DELETE`: File deleted
- `DELETE_SELF`: Watched file/directory deleted
- `MOVE_SELF`: Watched file/directory moved
- `OPEN_PERM`: Permission to open file
- `ACCESS_PERM`: Permission to access file

Convenience combinations:
- `ALL_ACCESS_EVENTS`: All access-related events
- `ALL_MODIFY_EVENTS`: All modification-related events
- `ALL_EVENTS`: All events

#### EventFlags

Response flags for permission events:

- `ALLOW`: Allow the operation
- `DENY`: Deny the operation

### Event

Represents a fanotify event.

#### Properties

- `info: EventInfo`: Event metadata
- `raw_data: Vec<u8>`: Raw event data

#### Methods

- `is_access() -> bool`: Check if this is an access event
- `is_modify() -> bool`: Check if this is a modify event
- `is_open() -> bool`: Check if this is an open event
- `is_close() -> bool`: Check if this is a close event
- `is_create() -> bool`: Check if this is a create event
- `is_delete() -> bool`: Check if this is a delete event
- `is_move() -> bool`: Check if this is a move event
- `is_permission() -> bool`: Check if this is a permission event
- `description() -> String`: Get human-readable description
- `event_type() -> &'static str`: Get event type as string

### EventInfo

Event metadata.

#### Properties

- `fd: Option<i32>`: File descriptor associated with the event
- `path: Option<PathBuf>`: Path associated with the event
- `mask: MaskFlags`: Event mask
- `pid: u32`: Process ID that triggered the event
- `is_directory: bool`: Whether this is a directory event

## Error Handling

The crate provides comprehensive error handling through the `FanotifyError` enum:

```rust
#[derive(Error, Debug)]
pub enum FanotifyError {
    Io(io::Error),
    InvalidFlags { message: String },
    InvalidPath { path: String },
    NotSupported,
    PermissionDenied { message: String },
    InvalidEventData { message: String },
    BufferOverflow,
    WouldBlock,
    InvalidFd,
    SyscallFailed { syscall: &'static str, errno: i32 },
    NoEvents,
    InvalidMask { message: String },
}
```

## Running Examples

### Basic Monitor

```bash
cargo run --example basic_monitor /path/to/monitor
```

### Async Monitor

```bash
cargo run --example async_monitor /path/to/monitor
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgments

This crate is inspired by the Linux fanotify API and aims to provide a safe, idiomatic Rust interface for filesystem event monitoring. 