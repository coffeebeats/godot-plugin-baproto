# CLAUDE.md

> `baproto-godot`: A wrapper binary which implements a GDScript code generator for IR compiled by the `build-a-proto` schema compiler.

## Build & Test

```bash
cargo build          # Build
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt --check    # Check formatting
```

## Comment Headers

80-char delimited, centered text. Types: `Struct`, `Enum`, `Type`, `Trait`, `Fn`, `Impl`, `Mod`, `Macro`
Type name is the "context" (e.g. function name, trait being impl'ed, function being tested, etc.)

```rust
/* -------------------------------------------------------------------------- */
/*                              Struct: TypeName                              */
/* -------------------------------------------------------------------------- */

/* ------------------------------ Mod: SubName ------------------------------ */
mod subname;
pub use subname::*;
```

## Tests

Inline with all actions described by Given/When/Then comments (full sentences, end with period):

```rust
/* -------------------------------------------------------------------------- */
/*                                 Mod: Tests                                 */
/* -------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    /* ---------------------------- Tests: feature -------------------------- */

    #[test]
    fn test_feature_scenario_outcome() {
        // Given: Setup description.
        // Given: Another setup description.
        // When: Action description.
        // Then: Assertion description.
        // When: Another action description.
        // Then: Another assertion description.
    }
}
```

## Patterns

- Doc comments: `` `Name` `` or `` [`Name`] `` for links
- Doc comments (cont.): start doc comments with identifier:

    ```rust
    /// `ident` ...
    fn ident() {}
    ```
