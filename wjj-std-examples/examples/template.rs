//! Template rendering example
//!
//! Run with:
//! ```bash
//! cargo run -p wjj-std-examples --example template
//! ```

use wjj_std::fmt_tpl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WJJ-STD Template Examples ===\n");

    println!("1. Inline template:");
    let message = fmt_tpl!("Hello {{ name }}, you have {{ count }} messages.", {
        "name": "Alice",
        "count": 3
    })?;
    println!("   {}\n", message);

    println!("2. Nested data:");
    let order = fmt_tpl!("Order {{ id }} belongs to {{ user.name }}.", {
        "id": "A001",
        "user": {
            "name": "Bob"
        }
    })?;
    println!("   {}\n", order);

    println!("3. Repeated inline templates:");
    let first = fmt_tpl!("Hi {{ name }}", {
        "name": "Carol"
    })?;
    let second = fmt_tpl!("Hi {{ name }}", {
        "name": "Dave"
    })?;
    println!("   {}", first);
    println!("   {}\n", second);

    println!("4. Loop and condition:");
    let list = fmt_tpl!(
        r#"
{% if items %}
Items:
{% for item in items %}
- {{ item }}
{% endfor %}
{% else %}
No items
{% endif %}
"#,
        {
            "items": ["apple", "banana"]
        }
    )?;
    println!("{}", list.trim());

    Ok(())
}
