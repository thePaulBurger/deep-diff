//! A small crate to deeply diff `serde_json::Value` trees.
//!
//! # Example
//!
//! ```rust
//! use deep_diff::{deep_diff, Difference};
//! use serde_json::json;
//!
//! let a = json!({"name": "Alice"});
//! let b = json!({"name": "Bob"});
//! let diffs = deep_diff(&a, &b);
//! assert_eq!(diffs[0].path, "name");
//!

use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Difference {
    /// The path to the value that changed (e.g., `"name"` or `"items[0]"`).
    pub path: String,
    /// The value before the change (in the first input).
    pub before: Option<Value>,
    /// The value after the change (in the second input).
    pub after: Option<Value>,
}

// Determines if two json types are equivalent
fn same_json_type(a: &Value, b: &Value) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

fn recurse(a: &Value, b: &Value, differences: &mut Vec<Difference>, path: String) {
    if !same_json_type(a, b) {
        differences.push(Difference {
            path: path.clone(),
            before: Some(a.clone()),
            after: Some(b.clone()),
        });
        return;
    }
    match a {
        // Deals with primitive types
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            if a != b {
                differences.push(Difference {
                    path: path.clone(),
                    before: Some(a.clone()),
                    after: Some(b.clone()),
                })
            }
        }
        // Deals with arrays
        Value::Array(a_values) => {
            let b_values = b.as_array().unwrap();
            for i in 0..a_values.len().max(b_values.len()) {
                let va = a_values.get(i).unwrap_or(&Value::Null);
                let vb = b_values.get(i).unwrap_or(&Value::Null);
                recurse(va, vb, differences, format!("{}[{}]", path, i));
            }
        }
        // Deals with objects
        Value::Object(map) => {
            for (ak, av) in map {
                match b.get(ak) {
                    Some(bv) => {
                        let full_path = if path.is_empty() {
                            ak.to_string()
                        } else {
                            format!("{}.{}", path, ak)
                        };
                        recurse(av, bv, differences, full_path);
                    }
                    None => differences.push(Difference {
                        path: format!("{}", ak),
                        before: Some(av.clone()),
                        after: None,
                    }),
                }
            }
            for (bk, bv) in b.as_object().unwrap() {
                if !map.contains_key(bk) {
                    let full_path = if path.is_empty() {
                        bk.to_string()
                    } else {
                        format!("{}.{}", path, bk)
                    };
                    differences.push(Difference {
                        path: full_path,
                        before: None,
                        after: Some(bv.clone()),
                    });
                }
            }
        }
    }
}

/// Computes the differences between two JSON values.
pub fn deep_diff(a: &Value, b: &Value) -> Vec<Difference> {
    let mut differences = Vec::new();
    recurse(a, b, &mut differences, "".to_string());
    differences
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Test that no differences are found when comparing identical primitive JSON values.
    #[test]
    fn test_no_change() {
        let a = json!("Alice");
        let result = deep_diff(&a, &a);
        assert!(result.is_empty());
    }

    /// Test difference at the top-level between two string JSON values.
    #[test]
    fn test_top_level_change() {
        let a = json!("Alice");
        let b = json!("Bob");
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "".to_string(),
                before: Some(json!("Alice")),
                after: Some(json!("Bob")),
            }]
        );
    }

    // ======================
    // Array Comparison Tests
    // ======================

    /// Test difference between numeric elements in an array.
    #[test]
    fn test_array_number_change() {
        let a = json!([1, 2]);
        let b = json!([1, 3]);
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "[1]".to_string(),
                before: Some(json!(2)),
                after: Some(json!(3)),
            }]
        );
    }

    /// Test difference between string elements in an array.
    #[test]
    fn test_array_string_change() {
        let a = json!(["Alice", "Bob"]);
        let b = json!(["Alice", "Hob"]);
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "[1]".to_string(),
                before: Some(json!("Bob")),
                after: Some(json!("Hob")),
            }]
        );
    }

    /// Test difference when arrays are of unequal length.
    #[test]
    fn test_array_unequal_length() {
        let a = json!([1, 2]);
        let b = json!([1]);
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "[1]".to_string(),
                before: Some(json!(2)),
                after: Some(Value::Null),
            }]
        );
    }

    // ======================
    // Object Comparison Tests
    // ======================

    /// Test that no differences are found when comparing identical maps.
    #[test]
    fn test_compare_map_same() {
        let a = json!({"name": "Bob", "age": 25});
        let result = deep_diff(&a, &a);
        assert!(result.is_empty());
    }

    /// Test difference in a single field of a map.
    #[test]
    fn test_compare_map_different() {
        let a = json!({"name": "Bob", "age": 25});
        let b = json!({"name": "Bob", "age": 26});
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "age".to_string(),
                before: Some(json!(25)),
                after: Some(json!(26)),
            }]
        );
    }

    // ======================
    // Deep Nested JSON Tests
    // ======================

    /// Test difference in deeply nested object fields.
    #[test]
    fn test_deep_nested_object() {
        let a = json!({ "person": { "name": { "first": "Alice" } } });
        let b = json!({ "person": { "name": { "first": "Bob" } } });
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "person.name.first".to_string(),
                before: Some(json!("Alice")),
                after: Some(json!("Bob")),
            }]
        );
    }

    /// Test difference in deeply nested array elements.
    #[test]
    fn test_deep_nested_array() {
        let a = json!({ "person": { "name": { "first": [1, 2, 3] } } });
        let b = json!({ "person": { "name": { "first": [1, 2, 4] } } });
        let result = deep_diff(&a, &b);
        assert_eq!(
            result,
            vec![Difference {
                path: "person.name.first[2]".to_string(),
                before: Some(json!(3)),
                after: Some(json!(4)),
            }]
        );
    }
}
