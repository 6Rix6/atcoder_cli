use std::collections::BTreeMap;

/// Expands `${VAR}` placeholders using the given variable map.
///
/// Variables are replaced in deterministic order (`BTreeMap`), and unknown
/// variables are left untouched with a warning.
pub fn expand_template(template: &str, vars: &BTreeMap<&str, &str>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
    }

    let mut rest = result.as_str();
    while let Some(start) = rest.find("${") {
        if let Some(end) = rest[start..].find('}') {
            let var_name = &rest[start + 2..start + end];
            warn_ln!(
                "unknown template variable '${{{}}}' in \"{}\"",
                var_name,
                template
            );
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars<'a>(pairs: &[(&'a str, &'a str)]) -> BTreeMap<&'a str, &'a str> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn replaces_known_variables() {
        let v = vars(&[("TASK_ID", "abc350_a"), ("CONTEST_ID", "abc350")]);

        assert_eq!(expand_template("${TASK_ID}.cpp", &v), "abc350_a.cpp");
        assert_eq!(
            expand_template("g++ ${CONTEST_ID}/${TASK_ID}", &v),
            "g++ abc350/abc350_a"
        );
        // 複数変数・複数回の出現
        assert_eq!(
            expand_template("${CONTEST_ID}_${TASK_ID}.log", &v),
            "abc350_abc350_a.log"
        );
    }

    #[test]
    fn leaves_unknown_variables_untouched() {
        let v = vars(&[("TASK_ID", "abc350_a")]);

        assert_eq!(
            expand_template("./build/${UNKNOWN}.out", &v),
            "./build/${UNKNOWN}.out"
        );
    }

    #[test]
    fn template_without_variables_is_unchanged() {
        assert_eq!(expand_template("./solution", &vars(&[])), "./solution");
        // 閉じ括弧が無い場合も壊さない
        assert_eq!(
            expand_template("weird ${UNCLOSED", &vars(&[])),
            "weird ${UNCLOSED"
        );
    }

    #[test]
    fn replacement_order_is_deterministic() {
        // 値自身が ${...} を含む場合も、キー順の置換で結果が一意になる
        let v = vars(&[("A", "${B}"), ("B", "x")]);
        assert_eq!(expand_template("${A}${B}", &v), "xx");
    }
}
