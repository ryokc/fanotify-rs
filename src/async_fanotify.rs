#[cfg(feature = "tokio")]
use std::path::Path;
#[cfg(feature = "tokio")]
use std::collections::HashMap;
#[cfg(feature = "tokio")]
use std::pin::Pin;
#[cfg(feature = "tokio")]
use std::task::{Context, Poll};
#[cfg(feature = "tokio")]
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(feature = "tokio")]
use tokio::fs::File;
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;

use crate::{
    error::{FanotifyError, Result},
    flags::FanotifyFlags,
};

#[cfg(feature = "tokio")]
use crate::{
    flags::{MaskFlags, EventFlags},
    event::Event,
    linux::{fanotify_init, fanotify_mark, fanotify_response, FAN_MARK_ADD, FAN_MARK_REMOVE, errno},
};

/// An asynchronous fanotify instance for monitoring filesystem events
#[cfg(feature = "tokio")]
pub struct AsyncFanotify {
    /// The file descriptor for the fanotify instance
    fd: File,
    /// Buffer for reading events
    buffer: Vec<u8>,
    /// Watched paths and their masks
    watched_paths: HashMap<std::path::PathBuf, MaskFlags>,
}

#[cfg(feature = "tokio")]
impl AsyncFanotify {
    /// Create a new asynchronous fanotify instance with default flags
    pub fn new() -> Result<Self> {
        Self::with_flags(FanotifyFlags::default())
    }

    /// Create a new asynchronous fanotify instance with custom flags
    pub fn with_flags(flags: FanotifyFlags) -> Result<Self> {
        let result = unsafe {
            fanotify_init(
                flags.bits(),
                libc::O_RDONLY as u32,
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        // SAFETY: result is a valid file descriptor
        let std_file = unsafe { std::fs::File::from_raw_fd(result) };
        let fd = File::from_std(std_file);

        Ok(Self {
            fd,
            buffer: vec![0u8; 4096],
            watched_paths: HashMap::new(),
        })
    }

    /// Add a watch for a path with the specified mask
    pub async fn add_watch<P: AsRef<Path>>(&mut self, path: P, mask: MaskFlags) -> Result<()> {
        let path = path.as_ref();
        
        // Convert path to C string
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(FanotifyError::invalid_path(path.to_string_lossy().to_string())),
        };

        let result = unsafe {
            fanotify_mark(
                self.fd.as_raw_fd(),
                FAN_MARK_ADD,
                mask.bits(),
                libc::AT_SYMLINK_NOFOLLOW,
                path_cstr.as_ptr(),
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        self.watched_paths.insert(path.to_path_buf(), mask);
        Ok(())
    }

    /// Remove a watch for a path
    pub async fn remove_watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(FanotifyError::invalid_path(path.to_string_lossy().to_string())),
        };

        let mask = self.watched_paths.get(path).copied().unwrap_or(MaskFlags::empty());

        let result = unsafe {
            fanotify_mark(
                self.fd.as_raw_fd(),
                FAN_MARK_REMOVE,
                mask.bits(),
                libc::AT_SYMLINK_NOFOLLOW,
                path_cstr.as_ptr(),
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        self.watched_paths.remove(path);
        Ok(())
    }

    /// Read a single event asynchronously
    pub async fn read_event(&mut self) -> Result<Option<Event>> {
        let bytes_read = match self.fd.read(&mut self.buffer).await {
            Ok(n) => n,
            Err(e) => return Err(FanotifyError::Io(e)),
        };

        if bytes_read == 0 {
            return Ok(None);
        }

        let event = Event::from_raw_data(&self.buffer[..bytes_read])?;
        Ok(Some(event))
    }

    /// Read all available events asynchronously
    pub async fn read_events(&mut self) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        
        loop {
            match self.read_event().await? {
                Some(event) => events.push(event),
                None => break,
            }
        }
        
        Ok(events)
    }

    /// Get the next event (returns None when no events are available)
    pub async fn next_event(&mut self) -> Result<Option<Event>> {
        self.read_event().await
    }

    /// Wait for the next event (blocks until an event is available)
    pub async fn wait_for_event(&mut self) -> Result<Event> {
        loop {
            if let Some(event) = self.read_event().await? {
                return Ok(event);
            }
            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    /// Respond to a permission event asynchronously
    pub async fn respond(&mut self, event: &Event, response: EventFlags) -> Result<()> {
        if !event.is_permission() {
            return Err(FanotifyError::invalid_event_data("Event is not a permission event"));
        }

        let fd = event.info.fd.ok_or_else(|| {
            FanotifyError::invalid_event_data("Permission event has no file descriptor")
        })?;

        let response_struct = fanotify_response {
            fd,
            response: response.bits(),
        };

        let result = unsafe {
            libc::write(
                self.fd.as_raw_fd(),
                &response_struct as *const _ as *const libc::c_void,
                std::mem::size_of::<fanotify_response>(),
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        Ok(())
    }

    /// Allow a permission event asynchronously
    pub async fn allow(&mut self, event: &Event) -> Result<()> {
        self.respond(event, EventFlags::ALLOW).await
    }

    /// Deny a permission event asynchronously
    pub async fn deny(&mut self, event: &Event) -> Result<()> {
        self.respond(event, EventFlags::DENY).await
    }

    /// Get the list of watched paths
    pub fn watched_paths(&self) -> &HashMap<std::path::PathBuf, MaskFlags> {
        &self.watched_paths
    }

    /// Check if a path is being watched
    pub fn is_watched<P: AsRef<Path>>(&self, path: P) -> bool {
        self.watched_paths.contains_key(path.as_ref())
    }

    /// Get the mask for a watched path
    pub fn get_mask<P: AsRef<Path>>(&self, path: P) -> Option<MaskFlags> {
        self.watched_paths.get(path.as_ref()).copied()
    }

    /// Set the buffer size for reading events
    pub fn set_buffer_size(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Create an event stream for use with tokio streams
    pub fn event_stream(&mut self) -> EventStream {
        EventStream { fanotify: self }
    }
}

#[cfg(feature = "tokio")]
impl Drop for AsyncFanotify {
    fn drop(&mut self) {
        // The file will be closed automatically when dropped
    }
}

#[cfg(feature = "tokio")]
impl std::os::unix::io::AsRawFd for AsyncFanotify {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.fd.as_raw_fd()
    }
}

/// A stream of fanotify events for use with tokio streams
#[cfg(feature = "tokio")]
#[allow(dead_code)]
pub struct EventStream<'a> {
    fanotify: &'a mut AsyncFanotify,
}

#[cfg(feature = "tokio")]
impl<'a> futures::Stream for EventStream<'a> {
    type Item = Result<Event>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // This is a simplified implementation. In a real implementation,
        // you would need to handle the async reading properly with tokio's
        // async I/O primitives.
        Poll::Pending
    }
}

#[cfg(feature = "tokio")]
impl<'a> EventStream<'a> {
    /// Create a new event stream
    pub fn new(fanotify: &'a mut AsyncFanotify) -> Self {
        Self { fanotify }
    }
}

// Placeholder struct when tokio feature is disabled
#[cfg(not(feature = "tokio"))]
pub struct AsyncFanotify;

#[cfg(not(feature = "tokio"))]
impl AsyncFanotify {
    pub fn new() -> Result<Self> {
        Err(FanotifyError::NotSupported)
    }

    pub fn with_flags(_flags: FanotifyFlags) -> Result<Self> {
        Err(FanotifyError::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_fanotify_creation() {
        let fanotify = AsyncFanotify::new();
        assert!(fanotify.is_ok());
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_add_watch() {
        let fanotify = AsyncFanotify::new();
        assert!(fanotify.is_ok());
        let mut fanotify = fanotify.unwrap();
        let temp_dir = tempdir().unwrap();
        
        let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY).await;
        assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
        assert!(fanotify.is_watched(temp_dir.path()));
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_remove_watch() {
        let fanotify = AsyncFanotify::new();
        assert!(fanotify.is_ok());
        let mut fanotify = fanotify.unwrap();
        let temp_dir = tempdir().unwrap();
        
        let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY).await;
        if let Err(e) = &result {
            eprintln!("add_watch failed: {:?}", e);
        }
        assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
        assert!(fanotify.is_watched(temp_dir.path()));
        
        let result = fanotify.remove_watch(temp_dir.path()).await;
        if let Err(e) = &result {
            eprintln!("remove_watch failed: {:?}", e);
        }
        assert!(result.is_ok(), "remove_watch failed: {:?}", result.err());
        assert!(!fanotify.is_watched(temp_dir.path()));
    }
} 