use std::{collections::HashMap, sync::LazyLock};

static MATHJAX_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (r"\lt", "<"),
        (r"\gt", ">"),
        (r"\le", r"\leq"),
        (r"\ge", r"\geq"),
    ])
});

pub fn replace_mathjax_aliases(s: &str) -> String {
    let mut result = s.to_string();
    for (from, to) in MATHJAX_ALIASES.iter() {
        result = result.replace(from, to);
    }
    result
}


