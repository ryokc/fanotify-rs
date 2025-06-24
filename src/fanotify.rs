use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::{
    error::{FanotifyError, Result},
    flags::{FanotifyFlags, MaskFlags, EventFlags},
    event::Event,
    linux::{fanotify_init, fanotify_mark, fanotify_response, FAN_MARK_ADD, FAN_MARK_REMOVE, errno},
};

/// A fanotify instance for monitoring filesystem events
pub struct Fanotify {
    /// The file descriptor for the fanotify instance
    fd: Option<File>,
    /// Buffer for reading events
    buffer: Vec<u8>,
    /// Watched paths and their masks
    watched_paths: HashMap<PathBuf, MaskFlags>,
}

impl Fanotify {
    /// Create a new fanotify instance with default flags
    pub fn new() -> Result<Self> {
        Self::with_flags(FanotifyFlags::default())
    }

    /// Create a new fanotify instance with custom flags
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

        let fd = unsafe { File::from_raw_fd(result) };
        Ok(Self {
            fd: Some(fd),
            buffer: vec![0u8; 4096],
            watched_paths: HashMap::new(),
        })
    }

    /// Add a watch for a path with the specified mask
    pub fn add_watch<P: AsRef<Path>>(&mut self, path: P, mask: MaskFlags) -> Result<()> {
        let path = path.as_ref();
        
        // Convert path to C string
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(FanotifyError::invalid_path(path.to_string_lossy().to_string())),
        };

        let result = unsafe {
            fanotify_mark(
                self.fd.as_ref().map(|f| f.as_raw_fd()).unwrap_or(-1),
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
    pub fn remove_watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        let path_cstr = match std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(FanotifyError::invalid_path(path.to_string_lossy().to_string())),
        };

        let mask = self.watched_paths.get(path).copied().unwrap_or(MaskFlags::empty());

        let result = unsafe {
            fanotify_mark(
                self.fd.as_ref().map(|f| f.as_raw_fd()).unwrap_or(-1),
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

    /// Read a single event
    pub fn read_event(&mut self) -> Result<Option<Event>> {
        let bytes_read = match self.fd.as_mut().map(|f| f.read(&mut self.buffer)) {
            Some(Ok(n)) => n,
            Some(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Some(Err(e)) => return Err(FanotifyError::Io(e)),
            None => return Ok(None),
        };

        if bytes_read == 0 {
            return Ok(None);
        }

        let event = Event::from_raw_data(&self.buffer[..bytes_read])?;
        Ok(Some(event))
    }

    /// Read all available events
    pub fn read_events(&mut self) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        
        loop {
            match self.read_event()? {
                Some(event) => events.push(event),
                None => break,
            }
        }
        
        Ok(events)
    }

    /// Get an iterator over events
    pub fn events(&mut self) -> EventIterator {
        EventIterator { fanotify: self }
    }

    /// Respond to a permission event
    pub fn respond(&self, event: &Event, response: EventFlags) -> Result<()> {
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
                self.fd.as_ref().map(|f| f.as_raw_fd()).unwrap_or(-1),
                &response_struct as *const _ as *const libc::c_void,
                std::mem::size_of::<fanotify_response>(),
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        Ok(())
    }

    /// Allow a permission event
    pub fn allow(&self, event: &Event) -> Result<()> {
        self.respond(event, EventFlags::ALLOW)
    }

    /// Deny a permission event
    pub fn deny(&self, event: &Event) -> Result<()> {
        self.respond(event, EventFlags::DENY)
    }

    /// Get the list of watched paths
    pub fn watched_paths(&self) -> &HashMap<PathBuf, MaskFlags> {
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
}

impl Drop for Fanotify {
    fn drop(&mut self) {
        // Let File's Drop handle closing the file descriptor
        let _ = self.fd.take();
    }
}

impl AsRawFd for Fanotify {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_ref().map(|f| f.as_raw_fd()).unwrap_or(-1)
    }
}

/// Iterator over fanotify events
pub struct EventIterator<'a> {
    fanotify: &'a mut Fanotify,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.fanotify.read_event() {
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> EventIterator<'a> {
    /// Create a new event iterator
    pub fn new(fanotify: &'a mut Fanotify) -> Self {
        Self { fanotify }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fanotify_creation() {
        let fanotify = Fanotify::new();
        assert!(fanotify.is_ok());
    }

    #[test]
    fn test_add_watch() {
        let fanotify = Fanotify::new();
        assert!(fanotify.is_ok());
        let mut fanotify = fanotify.unwrap();
        let temp_dir = tempdir().unwrap();
        
        let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY);
        assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
        assert!(fanotify.is_watched(temp_dir.path()));
    }

    #[test]
    fn test_remove_watch() {
        let fanotify = Fanotify::new();
        assert!(fanotify.is_ok());
        let mut fanotify = fanotify.unwrap();
        let temp_dir = tempdir().unwrap();
        
        let result = fanotify.add_watch(temp_dir.path(), MaskFlags::ACCESS | MaskFlags::MODIFY);
        assert!(result.is_ok(), "add_watch failed: {:?}", result.err());
        assert!(fanotify.is_watched(temp_dir.path()));
        
        let result = fanotify.remove_watch(temp_dir.path());
        assert!(result.is_ok(), "remove_watch failed: {:?}", result.err());
        assert!(!fanotify.is_watched(temp_dir.path()));
    }
} 