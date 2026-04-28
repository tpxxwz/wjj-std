pub use wjj_std_core::format_template_cached;
pub use wjj_std_core::format_template_once;

#[macro_export]
macro_rules! fmt_tpl_once {
    ($tpl:expr, $args:tt $(,)?) => {
        $crate::format_template_once($tpl, serde_json::json!($args))
    };
}

#[macro_export]
macro_rules! fmt_tpl {
    ($tpl:expr, $args:tt $(,)?) => {
        $crate::format_template_cached($tpl, serde_json::json!($args))
    };
}
