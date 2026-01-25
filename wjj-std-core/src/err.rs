use linkme::distributed_slice;
use minijinja::{Environment, UndefinedBehavior};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

pub struct TemplateRegistration {
    pub err_code: &'static str,
    pub err_tpl: &'static str,
}
#[distributed_slice]
pub static TEMPLATE_REGISTRATIONS: [TemplateRegistration] = [..];

pub struct ErrCodeRegistration {
    pub err_code: &'static str,
}

#[distributed_slice]
pub static ERR_CODE_REGISTRATIONS: [ErrCodeRegistration] = [..];

static ENV: Lazy<Arc<Environment<'static>>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    for reg in TEMPLATE_REGISTRATIONS {
        env.add_template(reg.err_code, reg.err_tpl)
            .unwrap_or_else(|e| panic!("template registration failed: {}", e));
    }
    Arc::new(env)
});

pub fn init() {
    let _ = &*ENV;
    let mut seen = HashSet::new();
    for reg in ERR_CODE_REGISTRATIONS {
        if !seen.insert(reg.err_code) {
            panic!("Duplicate err_code detected: {}", reg.err_code);
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
        write!(f, "{}", self.err_msg)
    }
}

impl fmt::Display for FmtErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = ENV
            .get_template(self.err_code)
            .and_then(|tpl| tpl.render(self.err_args.clone()))
            .unwrap_or_else(|_| {
                format!(
                    "[render failed. err_code: {}, err_tpl: {}, err_args: {}]",
                    self.err_code, self.err_tpl, self.err_args
                )
            });
        f.write_str(&output)
    }
}

impl Error for RawErr {}

impl Error for FmtErr {}
