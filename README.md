This package defines a macro that approximates the syntax for associated consts that is used in nightly Rust builds, but it turns the consts into functions so no nightly features are required. Therefore, we can have the appearance (but not the advantages) of associated consts in stable Rust.

Requires rust 1.3 or higher. This package will be irrelevant whenever associated consts get stabilized.

For documentation, run `cargo doc`.

