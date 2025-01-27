# Assembler
Simple customizible assembler

## CLI
### Building
The CLI can be built with [cargo](https://www.rust-lang.org/tools/install) using the following command:
```sh
cargo build --target <your-target-triple> --features=cli --bin assembler
```
If you dont know what is your target triple, please read [this](https://doc.rust-lang.org/cargo/appendix/glossary.html#target)

### Usage
```sh 
assembler -i source.asm -s synthax.toml -o output.mif
```
For more information, use `assembler -h`

## Defining synthaxes
The assembler synthaxes is defined in a [TOML](https://toml.io/) file containing the following tables:

### Symbols
Defines the synthax symbols
```toml
[symbols]
r0 = { value = 0, tags = ["reg"] }
sp = { value = 1, tags = ["sp"] }
```
- value: symbol value
- tags: sumbol tags, used to separate symbols and as instructions argument types

### Instructions
Defines the structure of the synthax instructions
```toml
[[instructions.mov]]
value = 0b1100110000000000
length = 16
arguments = [
    { type = "reg", index = 0, offset = 7, length = 3 },
    { type = "reg", index = 1, offset = 4, length = 3 },
]
documentation = "rx = ry"
```
- value: intruction value
- length: length of the instruction, if greater then the length of value, the remaining length will be assumed to be zeros
- arguments: intruction arguments
  - type: argument type, can be a data type or a symbol tag
  - index: argument index
  - offset: argument starting offset, with 0 being the least significant bit
  - length: argument length, if greater then the length of argument value, the remaining length will be assumed to be zeros
- documentation: optional key, used for documenting instructions

### Metadata
Optional table, used for storing addtional information about the synthax
```toml
[metadata]
name = "ICMC"
type = "instruction-set"
```
## WebAssembly
This project supports packaging for WebAssembly through [wasm-pack](https://github.com/rustwasm/wasm-pack)

### Building
```sh
wasm-pack build --target web --reference-types --weak-refs --release
```
