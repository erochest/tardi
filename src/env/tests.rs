use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_get_module_name_aliases() {
    let env = Environment::default();

    let result = env.get_module_name_aliases("std/vectors");

    assert_eq!(
        result,
        ["std/vectors".to_string(), "vectors".to_string()].to_vec()
    );
}

#[test]
fn test_expand_module_aliases() {
    let env = Environment::default();
    let segments = ["std/vectors".to_string(), "vectors".to_string()];
    let mut base = HashMap::new();
    base.insert("concat".to_string(), 4);
    base.insert(">string".to_string(), 7);
    base.insert("push!".to_string(), 13);

    let result = env.expand_module_aliases(&segments, &base);

    let mut keys = result.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    let expected = [
        ">string",
        "concat",
        "push!",
        "std/vectors/>string",
        "std/vectors/concat",
        "std/vectors/push!",
        "vectors/>string",
        "vectors/concat",
        "vectors/push!",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();
    assert_eq!(keys, expected);
    assert_eq!(result.get(">string"), Some(&7));
    assert_eq!(result.get("vectors/concat"), Some(&4));
    assert_eq!(result.get("std/vectors/push!"), Some(&13));
}
