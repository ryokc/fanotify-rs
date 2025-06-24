use bitflags::bitflags;
use crate::linux::{O_LARGEFILE, O_NOATIME};

bitflags! {
    /// Flags for fanotify initialization
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FanotifyFlags: u32 {
        /// Close-on-exec flag
        const CLOEXEC = libc::O_CLOEXEC as u32;
        
        /// Non-blocking flag
        const NONBLOCK = libc::O_NONBLOCK as u32;
        
        /// Read-only flag
        const RDONLY = libc::O_RDONLY as u32;
        
        /// Write-only flag
        const WRONLY = libc::O_WRONLY as u32;
        
        /// Read-write flag
        const RDWR = libc::O_RDWR as u32;
        
        /// Large file support
        const LARGEFILE = O_LARGEFILE as u32;
        
        /// No access time updates
        const NOATIME = O_NOATIME as u32;
        
        /// Directory-only operations
        const DIRECTORY = libc::O_DIRECTORY as u32;
        
        /// Follow symbolic links
        const NOFOLLOW = libc::O_NOFOLLOW as u32;
        
        /// Synchronous I/O
        const SYNC = libc::O_SYNC as u32;
        
        /// Data synchronization
        const DSYNC = libc::O_DSYNC as u32;
        
        /// Append mode
        const APPEND = libc::O_APPEND as u32;
        
        /// Create if not exists
        const CREAT = libc::O_CREAT as u32;
        
        /// Exclusive creation
        const EXCL = libc::O_EXCL as u32;
        
        /// Truncate on open
        const TRUNC = libc::O_TRUNC as u32;
    }
}

bitflags! {
    /// Event mask flags for fanotify
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MaskFlags: u64 {
        // Access events
        const ACCESS = 0x00000001;
        const MODIFY = 0x00000002;
        const ATTRIB = 0x00000004;
        const CLOSE_WRITE = 0x00000008;
        const CLOSE_NOWRITE = 0x00000010;
        const OPEN = 0x00000020;
        const MOVED_FROM = 0x00000040;
        const MOVED_TO = 0x00000080;
        const CREATE = 0x00000100;
        const DELETE = 0x00000200;
        const DELETE_SELF = 0x00000400;
        const MOVE_SELF = 0x00000800;
        
        // Permission events
        const OPEN_PERM = 0x00001000;
        const ACCESS_PERM = 0x00002000;
        
        // Directory events
        const ISDIR = 0x40000000;
        const UNMOUNT = 0x00002000;
        const Q_OVERFLOW = 0x00004000;
        const IGNORED = 0x00008000;
        
        // Special flags
        const ONLYDIR = 0x01000000;
        const DONT_FOLLOW = 0x02000000;
        const EXCL_UNLINK = 0x04000000;
        const MASK_ADD = 0x20000000;
        const IGNORED_MASK = 0x80000000;
        
        // Convenience combinations
        const ALL_ACCESS_EVENTS = Self::ACCESS.bits() | Self::MODIFY.bits() | Self::ATTRIB.bits() |
                                 Self::CLOSE_WRITE.bits() | Self::CLOSE_NOWRITE.bits() | Self::OPEN.bits();
        
        const ALL_MODIFY_EVENTS = Self::MODIFY.bits() | Self::ATTRIB.bits() | Self::CLOSE_WRITE.bits() |
                                 Self::CREATE.bits() | Self::DELETE.bits() | Self::DELETE_SELF.bits() |
                                 Self::MOVE_SELF.bits() | Self::MOVED_FROM.bits() | Self::MOVED_TO.bits();
        
        const ALL_EVENTS = Self::ALL_ACCESS_EVENTS.bits() | Self::ALL_MODIFY_EVENTS.bits() |
                          Self::OPEN_PERM.bits() | Self::ACCESS_PERM.bits() | Self::UNMOUNT.bits();
    }
}

bitflags! {
    /// Response flags for fanotify events
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EventFlags: u32 {
        /// Allow the operation
        const ALLOW = 0x01;
        
        /// Deny the operation
        const DENY = 0x02;
    }
}

impl Default for FanotifyFlags {
    fn default() -> Self {
        FanotifyFlags::RDONLY | FanotifyFlags::CLOEXEC
    }
}

impl Default for MaskFlags {
    fn default() -> Self {
        MaskFlags::ALL_EVENTS
    }
}

impl Default for EventFlags {
    fn default() -> Self {
        EventFlags::ALLOW
    }
}

impl MaskFlags {
    /// Check if the mask contains access events
    pub fn has_access_events(&self) -> bool {
        self.contains(MaskFlags::ACCESS | MaskFlags::OPEN | MaskFlags::ACCESS_PERM)
    }
    
    /// Check if the mask contains modify events
    pub fn has_modify_events(&self) -> bool {
        self.contains(MaskFlags::MODIFY | MaskFlags::ATTRIB | MaskFlags::CLOSE_WRITE | 
                     MaskFlags::CREATE | MaskFlags::DELETE | MaskFlags::DELETE_SELF | 
                     MaskFlags::MOVE_SELF | MaskFlags::MOVED_FROM | MaskFlags::MOVED_TO)
    }
    
    /// Check if the mask contains permission events
    pub fn has_permission_events(&self) -> bool {
        self.contains(MaskFlags::OPEN_PERM | MaskFlags::ACCESS_PERM)
    }
    
    /// Check if the mask is directory-only
    pub fn is_directory_only(&self) -> bool {
        self.contains(MaskFlags::ONLYDIR)
    }
    
    /// Check if the mask follows symbolic links
    pub fn follows_symlinks(&self) -> bool {
        !self.contains(MaskFlags::DONT_FOLLOW)
    }
} 