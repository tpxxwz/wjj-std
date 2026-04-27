use crate::template::{register_template, render_template};
use linkme::distributed_slice;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

pub struct ErrRegistration {
    pub err_code: &'static str,
    pub kind: ErrRegistrationKind,
}

pub enum ErrRegistrationKind {
    Raw { err_msg: &'static str },
    Template { err_tpl: &'static str },
}

#[distributed_slice]
pub static ERR_REGISTRATIONS: [ErrRegistration] = [..];

pub fn init() {
    let mut seen_err_codes = HashSet::new();
    for reg in ERR_REGISTRATIONS {
        if !seen_err_codes.insert(reg.err_code) {
            panic!("Duplicate err_code detected: {}", reg.err_code);
        }

        if let ErrRegistrationKind::Template { err_tpl } = reg.kind {
            register_template(reg.err_code, err_tpl)
                .unwrap_or_else(|e| panic!("template registration failed: {}", e));
        }
    }
}

#[derive(Debug)]
pub struct RawErr {
    pub err_code: &'static str,
    pub err_msg: &'static str,
}

#[derive(Debug)]
pub struct FmtErr {
    pub err_code: &'static str,
    pub err_tpl: &'static str,
    pub err_args: serde_json::Value,
}

impl fmt::Display for RawErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.err_msg)
    }
}

impl fmt::Display for FmtErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = render_template(self.err_code, self.err_tpl, self.err_args.clone())
            .unwrap_or_else(|e| {
                format!(
                    "[render failed. err_code: {}, err_tpl: {}, err_args: {}, cause: {}]",
                    self.err_code, self.err_tpl, self.err_args, e
                )
            });
        f.write_str(&output)
    }
}

impl Error for RawErr {}

impl Error for FmtErr {}
