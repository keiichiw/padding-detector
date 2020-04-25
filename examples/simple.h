#include<stdint.h>

struct test1 {
	uint8_t a;
	// <3-byte padding>
	uint32_t b;
	uint64_t c;
	uint8_t d;
	// <7-byte padding>
};
