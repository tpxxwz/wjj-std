use lru::LruCache;
use minijinja::{Environment, Error, UndefinedBehavior};
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

const REGISTERED_TEMPLATE_CAPACITY: usize = 1024;

static TEMPLATE_ID: AtomicU64 = AtomicU64::new(0);

fn next_template_id() -> String {
    format!("__t{}", TEMPLATE_ID.fetch_add(1, Ordering::Relaxed))
}

struct TemplateEngine {
    env: Environment<'static>,
    // key: source string, value: internal name in env
    registered: LruCache<String, String>,
}

static TEMPLATES: LazyLock<RwLock<TemplateEngine>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    RwLock::new(TemplateEngine {
        env,
        registered: LruCache::new(
            NonZeroUsize::new(REGISTERED_TEMPLATE_CAPACITY)
                .expect("registered template capacity must be non-zero"),
        ),
    })
});

pub fn format_template_once(source: &str, args: serde_json::Value) -> Result<String, Error> {
    TEMPLATES.read().env.render_str(source, args)
}

pub fn format_template_cached(source: &str, args: serde_json::Value) -> Result<String, Error> {
    if let Some(result) = try_render_from_cache(source, &args) {
        return result;
    }
    let name = ensure_template_registered(source)?;
    TEMPLATES.read().env.get_template(&name)?.render(args)
}

fn ensure_template_registered(source: &str) -> Result<String, Error> {
    let mut engine = TEMPLATES.write();
    if let Some(name) = engine.registered.get(source) {
        return Ok(name.clone());
    }
    let name = next_template_id();
    engine.env.add_template_owned(name.clone(), source.to_owned())?;
    if let Some((_, evicted_name)) = engine.registered.push(source.to_owned(), name.clone()) {
        engine.env.remove_template(&evicted_name);
    }
    Ok(name)
}

fn try_render_from_cache(
    source: &str,
    args: &serde_json::Value,
) -> Option<Result<String, Error>> {
    let engine = TEMPLATES.read();
    if let Some(name) = engine.registered.peek(source) {
        return Some(
            engine
                .env
                .get_template(name)
                .and_then(|t| t.render(args.clone())),
        );
    }
    None
}
