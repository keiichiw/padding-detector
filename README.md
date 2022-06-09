# padding-detector

A command line tool to detect implicit paddings that will be added in C structs and unions.

## Usage

```sh
$ cargo build --release
$ cat ./examples/simple.h
#include<stdint.h>

struct test1 {
        uint8_t a;
        // <3-byte padding>
        uint32_t b;
        uint64_t c;
        uint8_t d;
        // <7-byte padding>
};

union test2 {
        uint64_t a;
        char b[9]; // followed by 7-byte padding
};
$ ./target/release/padding-detector ./examples/simple.h
Checking `struct test1` (size=24)...
 Warning: Implicit padding was found: sum of fields: 14, struct size: 24
   Found: 3-byte padding before "b"
   Found: 7-byte padding at the end
Checking `union test2` (size=16)...
   Found: 7-byte padding is inserted
```

## How it works

The work flow consists of the following three steps:

1. Generates Rust FFI bindings to C structs in the given header by [bindgen].
2. Adds functions in the generated Rust file to check paddings.
3. Executes the generated Rust program in 2.

You can see the generated Rust codes by the following command:

```
# You'll find './out/bindings.rs' and './out/generated.rs' for step 1 and 2, respectively.
$ ./target/release/padding-detector ./examples/simple.h -o ./out/
```

The reason why it generates Rust bindings instead of processing C directly is it's easy to process
Rust's ASTs thanks to [syn].

[bindgen]: https://github.com/rust-lang/rust-bindgen
[syn]: https://github.com/dtolnay/syn

## Requirements

This tool depends on `libclang`, on which `bindgen` depends.
See [bindgen's doc](https://rust-lang.github.io/rust-bindgen/requirements.html).
