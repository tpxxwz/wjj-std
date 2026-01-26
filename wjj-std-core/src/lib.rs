#[cfg(feature = "error")]
mod err;

#[cfg(feature = "app")]
pub mod app;

#[cfg(feature = "error")]
pub use err::{
    ERR_CODE_REGISTRATIONS, ErrCodeRegistration, FmtErr, RawErr, TEMPLATE_REGISTRATIONS,
    TemplateRegistration,
};

#[cfg(feature = "app")]
pub use app::{Component, Registry};

#[cfg(feature = "error")]
#[ctor::ctor]
fn init_wjj_std_core() {
    err::init();
}
