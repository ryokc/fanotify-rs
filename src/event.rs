use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use crate::{FanotifyError, MaskFlags, Result};
use crate::linux::{fanotify_event_metadata, errno};

/// Information about a fanotify event
#[derive(Debug, Clone)]
pub struct EventInfo {
    /// The file descriptor associated with the event
    pub fd: Option<i32>,
    /// The path associated with the event (if available)
    pub path: Option<PathBuf>,
    /// The event mask
    pub mask: MaskFlags,
    /// The process ID that triggered the event
    pub pid: u32,
    /// Whether this is a directory event
    pub is_directory: bool,
}

/// A fanotify event
#[derive(Debug, Clone)]
pub struct Event {
    /// The event metadata
    pub info: EventInfo,
    /// Raw event data
    pub raw_data: Vec<u8>,
}

impl Event {
    /// Create a new event from raw fanotify data
    pub fn from_raw_data(data: &[u8]) -> Result<Self> {
        if data.len() < std::mem::size_of::<fanotify_event_metadata>() {
            return Err(FanotifyError::invalid_event_data("Data too short"));
        }

        let metadata = unsafe {
            &*(data.as_ptr() as *const fanotify_event_metadata)
        };

        let mask = MaskFlags::from_bits(metadata.mask)
            .ok_or_else(|| FanotifyError::invalid_mask("Invalid mask bits"))?;

        let mut info = EventInfo {
            fd: if metadata.fd >= 0 { Some(metadata.fd) } else { None },
            path: None,
            mask,
            pid: metadata.pid as u32,
            is_directory: mask.contains(MaskFlags::ISDIR),
        };

        // Try to get the path from the file descriptor
        if let Some(fd) = info.fd {
            info.path = Self::get_path_from_fd(fd).ok();
        }

        Ok(Event {
            info,
            raw_data: data.to_vec(),
        })
    }

    /// Get the path from a file descriptor
    fn get_path_from_fd(fd: i32) -> Result<PathBuf> {
        let mut buf = [0u8; libc::PATH_MAX as usize];
        
        let result = unsafe {
            libc::readlink(
                format!("/proc/self/fd/{}", fd).as_ptr() as *const i8,
                buf.as_mut_ptr() as *mut i8,
                buf.len() - 1,
            )
        };

        if result < 0 {
            return Err(FanotifyError::from(errno()));
        }

        let path_str = &buf[..result as usize];
        let path = OsStr::from_bytes(path_str);
        
        Ok(PathBuf::from(path))
    }

    /// Check if this is an access event
    pub fn is_access(&self) -> bool {
        self.info.mask.contains(MaskFlags::ACCESS)
    }

    /// Check if this is a modify event
    pub fn is_modify(&self) -> bool {
        self.info.mask.contains(MaskFlags::MODIFY)
    }

    /// Check if this is an open event
    pub fn is_open(&self) -> bool {
        self.info.mask.contains(MaskFlags::OPEN)
    }

    /// Check if this is a close event
    pub fn is_close(&self) -> bool {
        self.info.mask.contains(MaskFlags::CLOSE_WRITE | MaskFlags::CLOSE_NOWRITE)
    }

    /// Check if this is a create event
    pub fn is_create(&self) -> bool {
        self.info.mask.contains(MaskFlags::CREATE)
    }

    /// Check if this is a delete event
    pub fn is_delete(&self) -> bool {
        self.info.mask.contains(MaskFlags::DELETE | MaskFlags::DELETE_SELF)
    }

    /// Check if this is a move event
    pub fn is_move(&self) -> bool {
        self.info.mask.contains(MaskFlags::MOVED_FROM | MaskFlags::MOVED_TO | MaskFlags::MOVE_SELF)
    }

    /// Check if this is a permission event
    pub fn is_permission(&self) -> bool {
        self.info.mask.contains(MaskFlags::OPEN_PERM | MaskFlags::ACCESS_PERM)
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if self.is_access() {
            parts.push("access");
        }
        if self.is_modify() {
            parts.push("modify");
        }
        if self.is_open() {
            parts.push("open");
        }
        if self.is_close() {
            parts.push("close");
        }
        if self.is_create() {
            parts.push("create");
        }
        if self.is_delete() {
            parts.push("delete");
        }
        if self.is_move() {
            parts.push("move");
        }
        if self.is_permission() {
            parts.push("permission");
        }

        if parts.is_empty() {
            parts.push("unknown");
        }

        format!("{} event", parts.join(", "))
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        if self.is_access() {
            "ACCESS"
        } else if self.is_modify() {
            "MODIFY"
        } else if self.is_open() {
            "OPEN"
        } else if self.is_close() {
            "CLOSE"
        } else if self.is_create() {
            "CREATE"
        } else if self.is_delete() {
            "DELETE"
        } else if self.is_move() {
            "MOVE"
        } else if self.is_permission() {
            "PERMISSION"
        } else {
            "UNKNOWN"
        }
    }
}

impl EventInfo {
    /// Create a new event info structure
    pub fn new(mask: MaskFlags, pid: u32) -> Self {
        Self {
            fd: None,
            path: None,
            mask,
            pid,
            is_directory: mask.contains(MaskFlags::ISDIR),
        }
    }

    /// Set the file descriptor
    pub fn with_fd(mut self, fd: i32) -> Self {
        self.fd = Some(fd);
        self
    }

    /// Set the path
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Get the path as a string, if available
    pub fn path_str(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.to_str())
    }

    /// Get the filename, if available
    pub fn filename(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str())
    }
} 