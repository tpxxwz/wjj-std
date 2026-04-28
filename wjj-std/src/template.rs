pub use wjj_std_core::format_named_template;

/// Formats a MiniJinja template through the registered template cache.
#[macro_export]
macro_rules! fmt_tpl {
    ($tpl:expr, $args:tt $(,)?) => {
        $crate::format_named_template($tpl, $tpl, serde_json::json!($args))
    };
}
