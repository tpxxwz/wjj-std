//! # WJJ Standard Library
//!
//! WJJ's comprehensive standard library for Rust projects, providing a unified
//! toolkit with error handling, utilities, and more.
//!
//! ## Features
//!
//! - **Error Handling**: Template-based error generation with automatic error code management
//! - **String Utils**: String manipulation utilities (coming soon)
//! - **HTTP Utils**: HTTP client utilities (coming soon)
//! - **JSON Utils**: JSON processing utilities (coming soon)
//! - **Time Utils**: Time and date utilities (coming soon)
//!
//! ## Quick Start
//!
//! ### Error Handling
//!
//! ```rust
//! use wjj_std::{fmt_err, raw_err, FmtErr, RawErr};
//! use serde_json::json;
//!
//! // Template-based formatted error
//! #[derive(fmt_err)]
//! #[err_code_prefix = "001"]
//! pub enum UserErrors {
//!     #[error(err_code = "00001", err_tpl = "User {{ name }} not found")]
//!     UserNotFound,
//!
//!     #[error(err_code = "00002", err_tpl = "Invalid email: {{ email }}")]
//!     InvalidEmail,
//! }
//!
//! // Fixed message error
//! #[derive(raw_err)]
//! #[err_code_prefix = "002"]
//! pub enum SystemErrors {
//!     #[error(err_code = "00001", err_msg = "Database connection failed")]
//!     DbConnectionFailed,
//!
//!     #[error(err_code = "00002", err_msg = "Configuration error")]
//!     ConfigError,
//! }
//!
//! fn main() {
//!     // Use formatted error
//!     let err = UserErrors::UserNotFound.to_err(json!({
//!         "name": "Alice"
//!     }));
//!     println!("Error: {}", err);  // Error: User Alice not found
//!
//!     // Use raw error
//!     let err = SystemErrors::DbConnectionFailed.to_err();
//!     println!("Error: {}", err);  // Error: Database connection failed
//! }
//! ```
//!
//! ## Architecture
//!
//! This library uses a facade pattern with three internal crates:
//! - `wjj-std`: Public API (this crate)
//! - `wjj-std-core`: Core implementation
//! - `wjj-std-macros`: Procedural macros
//!
//! ## Feature Flags
//!
//! - `error` (default): Error handling functionality
//! - `string`: String utilities (coming soon)
//! - `http`: HTTP utilities (coming soon)
//! - `json`: JSON utilities (coming soon)
//! - `time`: Time utilities (coming soon)
//! - `full`: Enable all features
//!
//! ## Error Code System
//!
//! Error codes follow an 8-digit format: `PPPNNNNN`
//! - `PPP`: Prefix (3 digits) - Module identifier
//! - `NNNNN`: Number (5 digits) - Specific error identifier
//!
//! Example: `00100001` = Module `001`, Error `00001`

#![doc(html_root_url = "https://docs.rs/wjj-std/0.1.0")]
#![deny(missing_docs)]

// 为了让宏生成的代码能找到 ::wjj_std:: 路径
extern crate self as wjj_std;

// ========== Error Handling ==========
#[cfg(feature = "error")]
mod error;

#[cfg(feature = "error")]
pub use error::*;

// ========== Application Framework ==========
#[cfg(feature = "app")]
mod app;

#[cfg(feature = "app")]
pub use app::*;

// ========== Future Modules (Placeholder) ==========

#[cfg(feature = "string")]
/// String utilities module (coming soon)
pub mod string {
    //! String manipulation utilities
}

#[cfg(feature = "http")]
/// HTTP utilities module (coming soon)
pub mod http {
    //! HTTP client and server utilities
}

#[cfg(feature = "json")]
/// JSON utilities module (coming soon)
pub mod json {
    //! JSON processing utilities
}

#[cfg(feature = "time")]
/// Time utilities module (coming soon)
pub mod time {
    //! Time and date utilities
}
