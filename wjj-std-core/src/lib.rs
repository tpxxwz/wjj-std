// ========== Feature: error ==========
#[cfg(feature = "error")]
mod err;

#[cfg(feature = "error")]
pub use err::{ERR_REGISTRATIONS, ErrRegistration, ErrRegistrationKind, FmtErr, RawErr};

#[cfg(feature = "error")]
#[ctor::ctor]
fn init_wjj_std_core() {
    err::init();
}

// ========== Feature: app ==========
#[cfg(feature = "app")]
pub mod app;

#[cfg(feature = "app")]
pub use app::{Component, Registry};

// ========== Feature: template ==========
#[cfg(any(feature = "error", feature = "template"))]
mod template;

#[cfg(any(feature = "error", feature = "template"))]
pub use template::render_template;
