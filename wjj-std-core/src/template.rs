use lru::LruCache;
use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior};
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::LazyLock;

const REGISTERED_TEMPLATE_CAPACITY: usize = 1024;

struct TemplateEngine {
    env: Environment<'static>,
    sources: LruCache<String, String>,
}

static TEMPLATES: LazyLock<Mutex<TemplateEngine>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    Mutex::new(TemplateEngine {
        env,
        sources: LruCache::new(
            NonZeroUsize::new(REGISTERED_TEMPLATE_CAPACITY)
                .expect("registered template capacity must be non-zero"),
        ),
    })
});

pub fn format_named_template(
    name: &str,
    source: &str,
    args: serde_json::Value,
) -> Result<String, Error> {
    let mut engine = TEMPLATES.lock();

    match engine.sources.get(name) {
        Some(existing_source) if *existing_source != source => Err(template_conflict_error(name)),
        Some(_) => engine.env.get_template(name)?.render(args),
        None => {
            engine
                .env
                .add_template_owned(name.to_owned(), source.to_owned())?;
            if let Some((evicted_name, _)) = engine.sources.push(name.to_owned(), source.to_owned())
            {
                engine.env.remove_template(&evicted_name);
            }
            engine.env.get_template(name)?.render(args)
        }
    }
}

fn template_conflict_error(name: &str) -> Error {
    Error::new(
        ErrorKind::InvalidOperation,
        format!("template `{name}` already registered with different source"),
    )
}
