use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::LazyLock;

struct TemplateRegistry {
    env: Environment<'static>,
    sources: HashMap<&'static str, &'static str>,
}

static TEMPLATES: LazyLock<RwLock<TemplateRegistry>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    RwLock::new(TemplateRegistry {
        env,
        sources: HashMap::new(),
    })
});

pub fn render_template(
    name: &'static str,
    source: &'static str,
    args: serde_json::Value,
) -> Result<String, Error> {
    if let Some(output) = render_registered_template(name, source, args.clone())? {
        return Ok(output);
    }

    let mut registry = TEMPLATES.write();
    register_template_locked(&mut registry, name, source)?;

    registry.env.get_template(name)?.render(args)
}

#[allow(dead_code)]
pub(crate) fn register_template(name: &'static str, source: &'static str) -> Result<(), Error> {
    let mut registry = TEMPLATES.write();
    register_template_locked(&mut registry, name, source)
}

fn register_template_locked(
    registry: &mut TemplateRegistry,
    name: &'static str,
    source: &'static str,
) -> Result<(), Error> {
    match registry.sources.get(name) {
        Some(existing_source) if *existing_source != source => Err(template_conflict_error(name)),
        Some(_) => Ok(()),
        None => {
            registry.env.add_template(name, source)?;
            registry.sources.insert(name, source);
            Ok(())
        }
    }
}

fn render_registered_template(
    name: &'static str,
    source: &'static str,
    args: serde_json::Value,
) -> Result<Option<String>, Error> {
    let registry = TEMPLATES.read();

    match registry.sources.get(name) {
        Some(existing_source) if *existing_source == source => {
            Ok(Some(registry.env.get_template(name)?.render(args)?))
        }
        Some(_) => Err(template_conflict_error(name)),
        None => Ok(None),
    }
}

fn template_conflict_error(name: &str) -> Error {
    Error::new(
        ErrorKind::InvalidOperation,
        format!("template `{name}` already registered with different source"),
    )
}
