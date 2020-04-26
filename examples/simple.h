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
