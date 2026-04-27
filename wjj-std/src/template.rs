pub use wjj_std_core::render_template;

/// Renders a MiniJinja template through the global template cache.
///
/// The template is registered once by `name` and reused by later calls with the
/// same `name` and template source.
#[macro_export]
macro_rules! render_tpl {
    ($tpl:expr, $args:tt $(,)?) => {
        $crate::render_template(
            concat!(
                "__wjj_std_render_tpl::",
                file!(),
                ":",
                line!(),
                ":",
                column!()
            ),
            $tpl,
            serde_json::json!($args),
        )
    };
    ($name:expr, $tpl:expr, $args:tt $(,)?) => {
        $crate::render_template($name, $tpl, serde_json::json!($args))
    };
}
