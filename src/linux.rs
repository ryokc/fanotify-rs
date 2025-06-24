//! Linux-specific constants and types for fanotify
//!
//! This module contains constants and types that are specific to Linux's fanotify API
//! and may not be available in all libc implementations.

use libc::c_int;

// Fanotify event flags
pub const FAN_ACCESS: u64 = 0x00000001;
pub const FAN_MODIFY: u64 = 0x00000002;
pub const FAN_ATTRIB: u64 = 0x00000004;
pub const FAN_CLOSE_WRITE: u64 = 0x00000008;
pub const FAN_CLOSE_NOWRITE: u64 = 0x00000010;
pub const FAN_OPEN: u64 = 0x00000020;
pub const FAN_MOVED_FROM: u64 = 0x00000040;
pub const FAN_MOVED_TO: u64 = 0x00000080;
pub const FAN_CREATE: u64 = 0x00000100;
pub const FAN_DELETE: u64 = 0x00000200;
pub const FAN_DELETE_SELF: u64 = 0x00000400;
pub const FAN_MOVE_SELF: u64 = 0x00000800;
pub const FAN_OPEN_PERM: u64 = 0x00001000;
pub const FAN_ACCESS_PERM: u64 = 0x00002000;
pub const FAN_OPEN_EXEC_PERM: u64 = 0x00004000;
pub const FAN_OPEN_EXEC: u64 = 0x00008000;
pub const FAN_QUEUE_OVERFLOW: u64 = 0x00004000;
pub const FAN_FS_ERROR: u64 = 0x00008000;
pub const FAN_UNMOUNT: u64 = 0x00002000;
pub const FAN_ISDIR: u64 = 0x40000000;
pub const FAN_ONLYDIR: u64 = 0x01000000;
pub const FAN_DONT_FOLLOW: u64 = 0x02000000;
pub const FAN_EXCL_UNLINK: u64 = 0x04000000;
pub const FAN_MASK_ADD: u64 = 0x20000000;
pub const FAN_IGNORED_MASK: u64 = 0x80000000;
pub const FAN_IGNORED_SURV_MODIFY: u64 = 0x00002000;
pub const FAN_EVENT_ON_CHILD: u64 = 0x08000000;

// Fanotify mark flags
pub const FAN_MARK_ADD: u32 = 0x00000001;
pub const FAN_MARK_REMOVE: u32 = 0x00000002;
pub const FAN_MARK_DONT_FOLLOW: u32 = 0x00000004;
pub const FAN_MARK_ONLYDIR: u32 = 0x00000008;
pub const FAN_MARK_MOUNT: u32 = 0x00000010;
pub const FAN_MARK_IGNORED_MASK: u32 = 0x00000020;
pub const FAN_MARK_IGNORED_SURV_MODIFY: u32 = 0x00000040;
pub const FAN_MARK_FLUSH: u32 = 0x00000080;

// Fanotify response flags
pub const FAN_ALLOW: u32 = 0x01;
pub const FAN_DENY: u32 = 0x02;

// Fanotify init flags
pub const FAN_CLOEXEC: u32 = 0x00000001;
pub const FAN_NONBLOCK: u32 = 0x00000002;
pub const FAN_UNLIMITED_QUEUE: u32 = 0x00000010;
pub const FAN_UNLIMITED_MARKS: u32 = 0x00000020;
pub const FAN_REPORT_TID: u32 = 0x00000100;
pub const FAN_REPORT_FID: u32 = 0x00000200;
pub const FAN_REPORT_DIR_FID: u32 = 0x00000400;
pub const FAN_REPORT_NAME: u32 = 0x00000800;
pub const FAN_REPORT_DFID_NAME: u32 = 0x00000c00;

// Additional O_* flags that might be missing
pub const O_LARGEFILE: c_int = 0o100000;
pub const O_NOATIME: c_int = 0o1000000;

// Fanotify event metadata structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_metadata {
    pub event_len: u32,
    pub vers: u8,
    pub reserved: u8,
    pub metadata_len: u16,
    pub mask: u64,
    pub fd: i32,
    pub pid: i32,
}

// Fanotify response structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_response {
    pub fd: i32,
    pub response: u32,
}

// Fanotify info header structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_info_header {
    pub info_type: u8,
    pub pad: u8,
    pub len: u16,
}

// Fanotify info fid structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_info_fid {
    pub hdr: fanotify_event_info_header,
    pub fsid: libc::fsid_t,
    pub file_handle: [u8; 0],
}

// Fanotify info name structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_info_name {
    pub hdr: fanotify_event_info_header,
    pub dir_fh: [u8; 0],
}

// Fanotify info error structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_info_error {
    pub hdr: fanotify_event_info_header,
    pub error: i32,
    pub error_count: u64,
}

// Fanotify info pidfd structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fanotify_event_info_pidfd {
    pub hdr: fanotify_event_info_header,
    pub pidfd: i32,
}

// Fanotify info types
pub const FAN_EVENT_INFO_TYPE_FID: u8 = 1;
pub const FAN_EVENT_INFO_TYPE_DFID_NAME: u8 = 2;
pub const FAN_EVENT_INFO_TYPE_DFID: u8 = 3;
pub const FAN_EVENT_INFO_TYPE_PIDFD: u8 = 4;
pub const FAN_EVENT_INFO_TYPE_ERROR: u8 = 5;
pub const FAN_EVENT_INFO_TYPE_OLD_NAME: u8 = 6;
pub const FAN_EVENT_INFO_TYPE_NEW_DFID_NAME: u8 = 7;

// System call numbers (these may vary by architecture)
#[cfg(target_arch = "x86_64")]
pub const SYS_FANOTIFY_INIT: i32 = 300;
#[cfg(target_arch = "x86_64")]
pub const SYS_FANOTIFY_MARK: i32 = 301;

#[cfg(target_arch = "x86")]
pub const SYS_FANOTIFY_INIT: i32 = 337;
#[cfg(target_arch = "x86")]
pub const SYS_FANOTIFY_MARK: i32 = 338;

#[cfg(target_arch = "aarch64")]
pub const SYS_FANOTIFY_INIT: i32 = 262;
#[cfg(target_arch = "aarch64")]
pub const SYS_FANOTIFY_MARK: i32 = 263;

// Fallback for other architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
pub const SYS_FANOTIFY_INIT: i32 = 300;
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
pub const SYS_FANOTIFY_MARK: i32 = 301;

// Wrapper functions for system calls
pub unsafe fn fanotify_init(flags: u32, event_f_flags: u32) -> i32 {
    libc::syscall(SYS_FANOTIFY_INIT.into(), flags as i64, event_f_flags as i64) as i32
}

pub unsafe fn fanotify_mark(
    fanotify_fd: i32,
    flags: u32,
    mask: u64,
    dirfd: i32,
    pathname: *const i8,
) -> i32 {
    libc::syscall(
        SYS_FANOTIFY_MARK.into(),
        fanotify_fd as i64,
        flags as i64,
        mask as i64,
        dirfd as i64,
        pathname as i64,
    ) as i32
}

// Helper function to get errno
pub fn errno() -> i32 {
    #[cfg(target_os = "linux")]
    {
        unsafe { *libc::__errno_location() }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux systems, we'll use a fallback
        // This is mainly for compilation purposes
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
    }
} 
