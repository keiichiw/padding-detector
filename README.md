# struct-paddings-detector

Detect implicit paddings in C structs and unions.

```
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
$ cargo run
...
Checking `struct test1`...
3-byte padding before "b"
7-byte padding at the end
Checking `union test2`...
7-byte padding is inserted
```
