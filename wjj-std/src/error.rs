//! Error handling module

pub use wjj_std_core::{
    ErrCodeRegistration,
    ERR_CODE_REGISTRATIONS,
    FmtErr,
    RawErr,
    TemplateRegistration,
    TEMPLATE_REGISTRATIONS,
};

pub use wjj_std_macros::{fmt_err, raw_err};

/// Base raw error types for system-level errors
#[derive(raw_err)]
#[err_code_prefix = "999"]
pub enum BaseRawErrs {
    /// Generic system error
    #[error(err_code = "09999", err_msg = "System Error")]
    SysRawErr,
}

/// Base formatted error types for system-level errors with context
#[derive(fmt_err)]
#[err_code_prefix = "999"]
pub enum BaseFmtErrs {
    /// Generic system error with cause
    #[error(err_code = "99999", err_tpl = "System Error, cause: {{ cause }}")]
    SysFmtErr,
}
