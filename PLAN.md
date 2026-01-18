# Implementation Plan: Build-A-Proto Godot Plugin

## Current Status

✅ **Completed:**
1. Added minimal library exports to `build-a-proto` crate (ir, compile, generate modules)
2. Created `generate/` Rust crate scaffold with baproto dependency
   - Main CLI structure following baproto conventions
   - cmd/generate.rs command handler
   - GDScriptGenerator stub implementing Generator trait

## Remaining Tasks

### 1. Implement GDScript Language Generator

**File:** `generate/src/gdscript/language.rs`

Create a `GDScript` struct implementing the `Language<W>` trait:

```rust
pub struct GDScript {
    writer: CodeWriter,
}

impl Default for GDScript {
    fn default() -> Self {
        Self(CodeWriterBuilder::default()
            .comment_token("#".to_owned())
            .indent_token("\t".to_owned())  // GDScript uses tabs
            .newline_token("\n".to_owned())
            .build()
            .unwrap())
    }
}
```

**Implementation requirements:**
- `configure_writer()`: Map package name to `.gd` file path (e.g., `game.player` → `game/player.gd`)
- `pkg_begin()`: Write file header, class_name declaration, runtime imports
- `gen_msg_begin()`: Generate nested class definition with field declarations
- `gen_msg_end()`: Generate `encode()` and `static decode()` methods
- `gen_enum_begin()`/`gen_enum_end()`: Generate enum as constants
- `gen_field()`: Generate typed property with default value
- `gen_variant()`: Generate enum variant constant

**Type mapping (`type_name()` helper):**
- `Bool` → `bool`
- `Int{bits, signed}` → `int`
- `Float{bits}` → `float`
- `String` → `String`
- `Bytes` → `PackedByteArray`
- `Array{element}` → `Array[ElementType]`
- `Map{key, value}` → `Dictionary`
- `Message{descriptor}` → descriptor name
- `Enum{descriptor}` → `int` (enums are int constants in GDScript)

**Default values (`default_value()` helper):**
- `Bool` → `false`
- `Int` → `0`
- `Float` → `0.0`
- `String` → `""`
- `Bytes` → `PackedByteArray()`
- `Array` → `[]`
- `Map` → `{}`
- `Message` → `null`
- `Enum` → `0`

---

### 2. Implement Encode/Decode Code Generation

**File:** `generate/src/gdscript/codegen.rs`

Helper methods for the `GDScript` language implementation:

```rust
impl GDScript {
    /// Generate encode method body for a message
    fn gen_encode_method(&mut self, msg: &ir::Message, w: &mut W) -> Result<()> {
        for field in &msg.fields {
            self.gen_encode_field(field, w)?;
        }
    }

    /// Generate field encoding based on wire format
    fn gen_encode_field(&mut self, field: &ir::Field, w: &mut W) -> Result<()> {
        // Match on field.encoding.wire:
        // - WireFormat::Bits { count } → stream.write_bits(self.field, count)
        // - WireFormat::LengthPrefixed { prefix_bits } → stream.write_varint(...)
        // - WireFormat::Embedded → recursive message encoding
        // Apply transforms (ZigZag, Delta, FixedPoint) as needed
    }

    /// Generate decode method body for a message
    fn gen_decode_method(&mut self, msg: &ir::Message, w: &mut W) -> Result<()> {
        // Create instance
        // For each field, generate decode logic
        // Return instance
    }

    /// Generate field decoding based on wire format
    fn gen_decode_field(&mut self, field: &ir::Field, w: &mut W) -> Result<()> {
        // Match on field.encoding.wire (inverse of encode)
    }
}
```

**Wire format mapping:**
- `Bits{count}` → `write_bits(value, count)` / `read_bits(count)`
- `LengthPrefixed{prefix_bits}` → `write_varint()` / `read_varint()` for length, then data
- `Embedded` → Recursively call encode/decode on nested message

**Transform handling:**
- `ZigZag` → `write_zigzag()` / `read_zigzag()`
- `Delta` → Track previous value, encode/decode difference
- `FixedPoint{scale}` → Multiply/divide by scale factor

---

### 3. Implement GDScript Runtime

**File:** `addons/build_a_proto/runtime/bitstream.gd`

```gdscript
class_name BitStream
extends RefCounted

class Writer:
    extends RefCounted

    var _buffer: PackedByteArray
    var _bit_offset: int

    func write_bool(value: bool) -> void
    func write_bits(value: int, bit_count: int) -> void
    func write_zigzag(value: int, bit_count: int) -> void
    func write_varint(value: int) -> void
    func write_bytes(data: PackedByteArray) -> void
    func write_string(value: String, prefix_bits: int) -> void
    func to_bytes() -> PackedByteArray

class Reader:
    extends RefCounted

    var _buffer: PackedByteArray
    var _bit_offset: int

    func read_bool() -> bool
    func read_bits(bit_count: int) -> int
    func read_zigzag(bit_count: int) -> int
    func read_varint() -> int
    func read_bytes(count: int) -> PackedByteArray
    func read_string(prefix_bits: int) -> String
```

**Implementation notes:**
- Bit packing: Use bitwise operations to pack/unpack bits across byte boundaries
- Buffer management: Resize dynamically as needed
- ZigZag encoding: `(n << 1) ^ (n >> 63)` for encode, `(n >> 1) ^ -(n & 1)` for decode
- Varint: 7 bits per byte with continuation bit (MSB = 1 means more bytes follow)

---

### 4. Wire Up Generator in GDScriptGenerator

**File:** `generate/src/gdscript/mod.rs`

Update the `generate()` method to use the Language trait:

```rust
impl Generator for GDScriptGenerator {
    fn generate(&self, schema: &Schema) -> Result<GeneratorOutput, GeneratorError> {
        let mut gdscript_gen = gdscript::GDScript::default();
        let mut writers = HashMap::<PathBuf, StringWriter>::new();

        // Create writers for each package
        for pkg in &schema.packages {
            let path = gdscript_gen.configure_writer(Path::new("."), pkg)?;
            let mut w = StringWriter::default();
            w.open(&path)?;
            writers.insert(path, w);
        }

        gdscript_gen.gen_begin(schema, writers.iter_mut().collect())?;

        for pkg in &schema.packages {
            let path = gdscript_gen.configure_writer(Path::new("."), pkg)?;
            let w = writers.get_mut(&path).ok_or_else(|| ...)?;
            gdscript_gen.gen_pkg(schema, pkg, w)?;
        }

        gdscript_gen.gen_end(schema, writers.iter_mut().collect())?;

        // Convert writers to output
        let mut output = GeneratorOutput::default();
        for (path, writer) in writers {
            output.add(path, writer.as_str());
        }

        Ok(output)
    }
}
```

---

### 5. Create GDScript Import Plugin

**File:** `addons/build_a_proto/baproto_import_plugin.gd`

```gdscript
@tool
extends EditorImportPlugin

func _get_importer_name() -> String:
    return "build-a-proto.baproto"

func _get_visible_name() -> String:
    return "Build-A-Proto Schema"

func _get_recognized_extensions() -> PackedStringArray:
    return ["baproto"]

func _get_save_extension() -> String:
    return "gd"

func _get_resource_type() -> String:
    return "Script"

func _get_import_options(path: String, preset_index: int) -> Array[Dictionary]:
    return [
        {
            "name": "output_directory",
            "default_value": "res://generated/",
        },
    ]

func _import(source_file: String, save_path: String, options: Dictionary,
             platform_variants: Array[String], gen_files: Array[String]) -> Error:
    var generator_path := _find_generator_binary()
    if generator_path.is_empty():
        push_error("Generator binary not found for platform")
        return ERR_FILE_NOT_FOUND

    # Invoke generator: baproto-gdscript generate -o <out_dir> <source_file>
    var output_dir: String = options.get("output_directory", "res://generated/")
    var args := ["generate", "-o", output_dir, source_file]

    var exit_code := OS.execute(generator_path, args)
    if exit_code != 0:
        push_error("Generator failed with exit code: %d" % exit_code)
        return ERR_COMPILATION_FAILED

    # Find generated files and add to gen_files
    _collect_generated_files(output_dir, gen_files)

    return OK
```

**File:** `addons/build_a_proto/editor/platform.gd`

```gdscript
class_name BaprotoPlatform
extends RefCounted

static func get_generator_binary_name() -> String:
    var os_name := OS.get_name()
    var arch := ""

    if OS.has_feature("x86_64"):
        arch = "x86_64"
    elif OS.has_feature("arm64"):
        arch = "arm64"

    match os_name:
        "macOS":
            return "baproto-gdscript-darwin-%s" % arch
        "Linux":
            return "baproto-gdscript-linux-%s" % arch
        "Windows":
            return "baproto-gdscript-windows-%s.exe" % arch
        _:
            return ""
```

**File:** `addons/build_a_proto/plugin.gd`

```gdscript
@tool
extends EditorPlugin

var import_plugin: EditorImportPlugin

func _enter_tree():
    import_plugin = preload("baproto_import_plugin.gd").new()
    add_import_plugin(import_plugin)

func _exit_tree():
    remove_import_plugin(import_plugin)
    import_plugin = null
```

**File:** `addons/build_a_proto/plugin.cfg`

```ini
[plugin]
name="Build-A-Proto"
description="Import .baproto schema files and generate GDScript bindings"
author="coffeebeats"
version="0.1.0"
script="plugin.gd"
```

---

### 6. Set Up CI Pipeline

**Copy from build-a-proto:**
- `.github/actions/setup-rust/action.yml` (Rust toolchain + cross setup)

**File:** `.github/workflows/build-generator.yml` (for development/testing)

```yaml
name: Build Generator

on:
  push:
    paths:
      - 'generate/**'
      - '.github/workflows/build-generator.yml'
  pull_request:
    paths:
      - 'generate/**'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/setup-rust
        with:
          target: ${{ matrix.target }}
      - name: Build
        run: |
          cd generate
          cargo build --release --target ${{ matrix.target }}
```

**Update:** `.github/workflows/release-please.yml`

Add a job that runs after release PR is merged:
1. Build generator for all 5 platforms
2. Create staging directory
3. Copy `addons/build_a_proto/` to staging
4. Copy built binaries to `staging/addons/build_a_proto/bin/`
5. Zip staging directory as `build-a-proto-godot-v{version}.zip`
6. Upload to GitHub release

---

### 7. Add Tests

**File:** `tests/schemas/example.baproto`

```baproto
package test.basic;

message PlayerState {
    0: u16 position_x;
    1: u16 position_y;
    2: i8 velocity_x = zigzag;
    3: i8 velocity_y = zigzag;
    4: bool grounded;
}

enum Status {
    0: Inactive;
    1: Active;
    2: Pending;
}
```

**File:** `tests/test_bitstream.gd`

```gdscript
extends GutTest

const BitStream = preload("res://addons/build_a_proto/runtime/bitstream.gd")

func test_write_read_bool():
    var writer = BitStream.Writer.new()
    writer.write_bool(true)
    writer.write_bool(false)

    var bytes = writer.to_bytes()
    var reader = BitStream.Reader.new(bytes)

    assert_true(reader.read_bool())
    assert_false(reader.read_bool())

func test_write_read_bits():
    var writer = BitStream.Writer.new()
    writer.write_bits(42, 8)
    writer.write_bits(1023, 10)

    var bytes = writer.to_bytes()
    var reader = BitStream.Reader.new(bytes)

    assert_eq(reader.read_bits(8), 42)
    assert_eq(reader.read_bits(10), 1023)

func test_zigzag_encoding():
    var writer = BitStream.Writer.new()
    writer.write_zigzag(-64, 8)
    writer.write_zigzag(64, 8)

    var bytes = writer.to_bytes()
    var reader = BitStream.Reader.new(bytes)

    assert_eq(reader.read_zigzag(8), -64)
    assert_eq(reader.read_zigzag(8), 64)
```

**File:** `tests/test_generated.gd`

```gdscript
extends GutTest

const TestBasic = preload("res://generated/test/basic.gd")
const BitStream = preload("res://addons/build_a_proto/runtime/bitstream.gd")

func test_player_state_round_trip():
    var original = TestBasic.PlayerState.new()
    original.position_x = 100
    original.position_y = 200
    original.velocity_x = -5
    original.velocity_y = 10
    original.grounded = true

    var writer = BitStream.Writer.new()
    original.encode(writer)
    var bytes = writer.to_bytes()

    var reader = BitStream.Reader.new(bytes)
    var decoded = TestBasic.PlayerState.decode(reader)

    assert_eq(decoded.position_x, 100)
    assert_eq(decoded.position_y, 200)
    assert_eq(decoded.velocity_x, -5)
    assert_eq(decoded.velocity_y, 10)
    assert_true(decoded.grounded)
```

**Set up GUT (Godot Unit Test):**
- Add GUT as a plugin/addon (either via AssetLib or git submodule)
- Configure test runner in project settings

---

## Build Order

1. **Runtime first** - Implement `bitstream.gd` so we can test it independently
2. **Language trait** - Implement GDScript language generator with type mapping
3. **Codegen** - Add encode/decode generation logic
4. **Wire up** - Complete GDScriptGenerator integration
5. **Test generator** - Create test schemas, run generator manually, verify output
6. **Import plugin** - Implement EditorImportPlugin
7. **CI** - Set up build workflows
8. **Tests** - Add comprehensive GUT tests

---

## Testing Strategy

**Unit tests:**
- Bitstream operations (write/read for each primitive type)
- ZigZag encoding/decoding
- Varint encoding/decoding

**Integration tests:**
- Generate code from test schemas
- Round-trip encode/decode for messages
- Nested messages and enums
- Cross-package references

**Manual testing:**
- Import `.baproto` file in Godot editor
- Verify generated `.gd` files
- Use generated classes in a test scene
- Build project to ensure runtime-only code works

---

## File Structure Summary

```
build-a-proto-godot/
├── addons/build_a_proto/
│   ├── plugin.cfg
│   ├── plugin.gd
│   ├── baproto_import_plugin.gd
│   ├── runtime/
│   │   └── bitstream.gd
│   ├── bin/
│   │   └── .gitkeep
│   └── editor/
│       └── platform.gd
├── generate/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── cmd/
│       │   ├── mod.rs
│       │   └── generate.rs
│       └── gdscript/
│           ├── mod.rs
│           ├── language.rs
│           └── codegen.rs
├── tests/
│   ├── schemas/
│   │   └── example.baproto
│   ├── test_bitstream.gd
│   └── test_generated.gd
├── .github/
│   ├── actions/
│   │   └── setup-rust/
│   └── workflows/
│       ├── build-generator.yml
│       └── release-please.yml
└── PLAN.md (this file)
```