use std::io;
use thiserror::Error;

/// Result type for fanotify operations
pub type Result<T> = std::result::Result<T, FanotifyError>;

/// Errors that can occur during fanotify operations
#[derive(Error, Debug)]
pub enum FanotifyError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid flags provided
    #[error("Invalid flags: {message}")]
    InvalidFlags { message: String },

    /// Invalid path provided
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    /// Fanotify not supported by kernel
    #[error("Fanotify not supported by kernel")]
    NotSupported,

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Invalid event data
    #[error("Invalid event data: {message}")]
    InvalidEventData { message: String },

    /// Event buffer overflow
    #[error("Event buffer overflow")]
    BufferOverflow,

    /// Operation would block
    #[error("Operation would block")]
    WouldBlock,

    /// Invalid file descriptor
    #[error("Invalid file descriptor")]
    InvalidFd,

    /// System call failed
    #[error("System call failed: {syscall} - {errno}")]
    SyscallFailed { syscall: &'static str, errno: i32 },

    /// Event queue is empty
    #[error("Event queue is empty")]
    NoEvents,

    /// Invalid mask flags
    #[error("Invalid mask flags: {message}")]
    InvalidMask { message: String },
}

impl From<libc::c_int> for FanotifyError {
    fn from(errno: libc::c_int) -> Self {
        match errno {
            libc::ENOSYS => FanotifyError::NotSupported,
            libc::EACCES => FanotifyError::PermissionDenied {
                message: "Access denied".to_string(),
            },
            libc::EINVAL => FanotifyError::InvalidFlags {
                message: "Invalid flags".to_string(),
            },
            libc::ENOENT => FanotifyError::InvalidPath {
                path: "Path does not exist".to_string(),
            },
            libc::EAGAIN => FanotifyError::WouldBlock,
            libc::EBADF => FanotifyError::InvalidFd,
            libc::EOVERFLOW => FanotifyError::BufferOverflow,
            _ => FanotifyError::Io(io::Error::from_raw_os_error(errno)),
        }
    }
}

impl FanotifyError {
    /// Create a new permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        FanotifyError::PermissionDenied {
            message: message.into(),
        }
    }

    /// Create a new invalid flags error
    pub fn invalid_flags(message: impl Into<String>) -> Self {
        FanotifyError::InvalidFlags {
            message: message.into(),
        }
    }

    /// Create a new invalid path error
    pub fn invalid_path(path: impl Into<String>) -> Self {
        FanotifyError::InvalidPath {
            path: path.into(),
        }
    }

    /// Create a new system call failed error
    pub fn syscall_failed(syscall: &'static str, errno: i32) -> Self {
        FanotifyError::SyscallFailed { syscall, errno }
    }

    /// Create a new invalid event data error
    pub fn invalid_event_data(message: impl Into<String>) -> Self {
        FanotifyError::InvalidEventData {
            message: message.into(),
        }
    }

    /// Create a new invalid mask error
    pub fn invalid_mask(message: impl Into<String>) -> Self {
        FanotifyError::InvalidMask {
            message: message.into(),
        }
    }
} 