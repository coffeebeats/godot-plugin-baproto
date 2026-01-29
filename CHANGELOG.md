# Changelog

## 0.2.0 (2026-01-29)

## What's Changed
* chore!: update to Godot version `v4.6` by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/50
* fix(docs): Update versioning information in README.md by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/52


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.11...v0.2.0

## 0.1.11 (2026-01-27)

## What's Changed
* chore: update `baproto` to latest (fixes CI builds) by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/48


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.10...v0.1.11

## 0.1.10 (2026-01-27)

## What's Changed
* fix: upgrade `baproto` to latest by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/46


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.9...v0.1.10

## 0.1.9 (2026-01-26)

## What's Changed
* chore: delete leftover UID files by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/43
* chore: update transitive dependencies; use latest `baproto` version in CI by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/45


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.8...v0.1.9

## 0.1.8 (2026-01-26)

## What's Changed
* fix: optimize binaries for size by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/40
* fix(ci): check out correct branch when releasing by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/42


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.7...v0.1.8

## 0.1.7 (2026-01-26)

## What's Changed
* fix(ci): delete prior binaries when building by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/38


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.6...v0.1.7

## 0.1.6 (2026-01-26)

## What's Changed
* fix(ci): check out target branch before downloading build artifacts by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/36


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.5...v0.1.6

## 0.1.5 (2026-01-26)

## What's Changed
* feat: create a `baproto` runtime for reading and writing to bitstreams by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/28
* feat(generate): create GDScript source code generator by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/30
* feat(generate): implement serialization and deserialization of generated types by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/31
* feat(import): create an `EditorImportPlugin` for compiling schema files by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/32
* fix(plugin): address compilation issues in messages; remove runtime autoload by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/33
* refactor(gdscript): make AST nodes, generation more composable and maintainable by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/34
* fix(gdscript): allow early returns; fix assertion messages by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/35


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.4...v0.1.5

## 0.1.4 (2026-01-18)

## What's Changed
* chore: rename repository to `godot-plugin-baproto` by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/24
* fix(ci): omit Rust source when packaging addon by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/26
* chore(docs): update plugin name and description by @coffeebeats in https://github.com/coffeebeats/godot-plugin-baproto/pull/27


**Full Changelog**: https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.3...v0.1.4

## 0.1.3 (2026-01-18)

## What's Changed

* fix(ci): ignore correct target directory; upgrade `build-a-proto` actions versions by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/22>

**Full Changelog**: <https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.2...v0.1.3>

## 0.1.2 (2026-01-18)

## What's Changed

* fix(ci): mark `build` a dependency of `release-branch` by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/15>
* fix(generate): increment `generate` crate version alongside plugin by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/17>
* fix(ci): package Rust separately when publishing by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/18>
* refactor(generate): migrate `generate` crate into repository root by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/19>
* fix(ci): use correct target directory when compiling by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/21>

**Full Changelog**: <https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.1...v0.1.2>

## 0.1.1 (2026-01-17)

## What's Changed

* Bump tj-actions/changed-files from 47.0.0 to 47.0.1 by @dependabot[bot] in <https://github.com/coffeebeats/godot-plugin-baproto/pull/1>
* feat(generate): instantiate `generate` application crate by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/2>
* feat(generate): create CLI scaffold for generating GDScript files from schemas by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/4>
* chore(docs): add common issue templates by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/5>
* feat(ci): create workflow to check Rust code by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/6>
* chore: save updated icon import settings by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/7>
* fix(ci): correct errors with Rust workflow by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/8>
* chore(generate): add `AGENTS.md` documentation by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/9>
* fix(ci): ensure `cargo test` uses cached build artifacts by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/10>
* fix(ci): skip build cache when linting as `cargo clippy` ignores it by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/11>
* feat(ci): build and install `baproto-gdscript` upon release by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/12>
* chore: upgrade `baproto` to latest by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/13>
* fix(ci): correct build errors when compiling binary artifacts by @coffeebeats in <https://github.com/coffeebeats/godot-plugin-baproto/pull/14>

## New Contributors

* @dependabot[bot] made their first contribution in <https://github.com/coffeebeats/godot-plugin-baproto/pull/1>
* @coffeebeats made their first contribution in <https://github.com/coffeebeats/godot-plugin-baproto/pull/2>

**Full Changelog**: <https://github.com/coffeebeats/godot-plugin-baproto/compare/v0.1.0...v0.1.1>

## Changelog
