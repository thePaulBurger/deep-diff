# deep-diff

A small Rust crate to deeply diff `serde_json::Value` trees.

This crate helps you compare two JSON values recursively and returns a list of differences, showing exactly which parts have changed.

## Features

- Compare JSON objects, arrays, strings, numbers, booleans, and nulls
- Reports differences with precise JSON path notation
- Lightweight and easy to integrate

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep-diff = "0.1"
serde_json = "1.0"
```

## Example

```rust
use deep_diff::{deep_diff, Difference};
use serde_json::json;

fn main() {
    let a = json!({"name": "Alice", "age": 30});
    let b = json!({"name": "Bob", "age": 30});

    let diffs = deep_diff(&a, &b);

    for diff in diffs {
        println!("Changed at path '{}': from {:?} to {:?}", diff.path, diff.before, diff.after);
    }
}
```
