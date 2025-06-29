use fanotify_rs::{Fanotify, MaskFlags};
use tempfile::tempdir;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_functionality() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Create a fanotify instance
    let mut fanotify = Fanotify::new().unwrap();
    
    // Add a watch for the temporary directory
    let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY);
    assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
    
    // Verify the watch was added
    assert!(fanotify.is_watched(temp_dir.path()));
    
    // Create a file in the watched directory
    fs::write(&test_file, "test content").unwrap();
    
    // Give some time for the event to be generated
    let mut events = Vec::new();
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        let new_events = fanotify.read_events().unwrap();
        if !new_events.is_empty() {
            events.extend(new_events);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(!events.is_empty(), "No events received after file creation");
    
    // Check that we have a create event
    let has_create_event = events.iter().any(|event| event.is_create());
    assert!(has_create_event, "Expected to find a CREATE event");
    
    // Remove the watch
    fanotify.remove_watch(temp_dir.path()).unwrap();
    assert!(!fanotify.is_watched(temp_dir.path()));
}

#[test]
fn test_event_types() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    let mut fanotify = Fanotify::new().unwrap();
    let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY);
    assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
    
    // Create a file
    fs::write(&test_file, "test content").unwrap();
    thread::sleep(Duration::from_millis(50));
    
    // Modify the file
    fs::write(&test_file, "modified content").unwrap();
    thread::sleep(Duration::from_millis(50));
    
    // Read events
    let events = fanotify.read_events().unwrap();
    
    // Verify we have events
    assert!(!events.is_empty());
    
    // Check event types
    let event_types: Vec<&str> = events.iter()
        .map(|e| e.event_type())
        .collect();
    
    println!("Event types: {:?}", event_types);
    
    // We should have at least CREATE and MODIFY events
    assert!(event_types.contains(&"CREATE") || event_types.contains(&"MODIFY"));
}

#[test]
fn test_buffer_size() {
    let mut fanotify = Fanotify::new().unwrap();
    
    // Test default buffer size
    assert_eq!(fanotify.buffer_size(), 4096);
    
    // Test setting buffer size
    fanotify.set_buffer_size(8192);
    assert_eq!(fanotify.buffer_size(), 8192);
}

#[test]
fn test_watched_paths() {
    let temp_dir = tempdir().unwrap();
    let mut fanotify = Fanotify::new().unwrap();
    
    // Add multiple watches
    let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY);
    assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
    let result = fanotify.add_watch("/tmp", MaskFlags::ACCESS | MaskFlags::MODIFY);
    assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
    
    let watched_paths = fanotify.watched_paths();
    assert_eq!(watched_paths.len(), 2);
    assert!(watched_paths.contains_key(temp_dir.path()));
    assert!(watched_paths.contains_key(Path::new("/tmp")));
    
    // Test getting mask for a watched path
    let mask = fanotify.get_mask(temp_dir.path());
    assert_eq!(mask, Some(MaskFlags::ACCESS | MaskFlags::MODIFY));
} 