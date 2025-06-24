//! An idiomatic Rust wrapper for Linux fanotify
//!
//! This crate provides a safe and ergonomic interface to Linux's fanotify API,
//! which allows monitoring filesystem events for access, modification, and other
//! file operations.
//!
//! # Features
//!
//! - **Safe abstractions**: All unsafe operations are wrapped in safe Rust code
//! - **Error handling**: Comprehensive error types with detailed information
//! - **Async support**: Both synchronous and asynchronous event monitoring
//! - **Type safety**: Strongly typed flags and event types
//! - **Documentation**: Extensive documentation with examples
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```no_run
//! use fanotify_rs::{Fanotify, FanotifyFlags, EventFlags, MaskFlags};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut fanotify = Fanotify::new()?;
//!     
//!     // Monitor a directory for all events
//!     fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS)?;
//!     
//!     // Read events
//!     for event in fanotify.events() {
//!         println!("Event: {:?}", event);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Async usage
//!
//! ```no_run
//! use fanotify_rs::{AsyncFanotify, FanotifyFlags, MaskFlags};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut fanotify = AsyncFanotify::new()?;
//!     
//!     fanotify.add_watch("/tmp", MaskFlags::ALL_EVENTS).await?;
//!     
//!     while let Some(event) = fanotify.next_event().await? {
//!         println!("Async event: {:?}", event);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod flags;
pub mod event;
pub mod fanotify;
pub mod async_fanotify;
pub mod linux;

pub use error::{FanotifyError, Result};
pub use flags::{FanotifyFlags, MaskFlags, EventFlags};
pub use event::{Event, EventInfo};
pub use fanotify::Fanotify;
#[cfg(feature = "tokio")]
pub use async_fanotify::AsyncFanotify; 