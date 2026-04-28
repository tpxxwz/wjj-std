// ========== Feature: template ==========
#[cfg(feature = "template")]
mod template;

#[cfg(feature = "template")]
pub use template::format_named_template;

// ========== Feature: error ==========
#[cfg(feature = "error")]
mod error;

#[cfg(feature = "error")]
pub use error::{ERR_REGISTRATIONS, ErrRegistration, ErrRegistrationKind, FmtErr, RawErr};

#[cfg(feature = "error")]
#[ctor::ctor]
fn init_wjj_std_core() {
    error::init();
}

// ========== Feature: app ==========
#[cfg(feature = "app")]
pub mod app;

#[cfg(feature = "app")]
pub use app::{Component, Registry};
