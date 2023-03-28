#include <stdint.h>
#include <stdio.h>
#include <inttypes.h>

// A collection of C functions that are linked to Wellick
// and can be called from Wellick code. These functions act
// as the "builtins" of Wellick.

// The wellick "int" type is equivalent to a 32-bit unsigned integer
// So all integer functions use uint32_t in their parameters and return types

// Print a string to stdout
int print(uint64_t inp) {
    return printf("%" PRId64, inp);
}

int println_32(int32_t inp) {
    return printf("%d\n", inp);
}

// Print a string to stdout with a newline
int println(uint64_t inp) {
    return printf("%" PRId64 "\n", inp);
}

// Add two integers, return the result
uint32_t iadd(uint32_t a, uint16_t b) {
    return a + b;
}

// Subtract two integers, return the result
uint32_t isub(uint32_t a, uint16_t b) {
    return a - b;
}

// Multiply two integers, return the result
uint32_t imul(uint32_t a, uint16_t b) {
    return a * b;
}

// Divide two integers, return the result
uint32_t idiv(uint32_t a, uint16_t b) {
    return a / b;
}

// Check the equality of two integers, return 1 if true, 0 if false
uint32_t ieq (uint32_t a, uint16_t b) {
    return a == b;
}

// Check if the first integer is less than the second, return 1 if true, 0 if false
uint32_t ilt (uint32_t a, uint16_t b) {
    return a < b;
}

uint32_t ilteq (uint32_t a, uint16_t b) {
    return a <= b;
}

// Check if the first integer is greater than the second, return 1 if true, 0 if false
uint32_t igt (uint32_t a, uint16_t b) {
    return a > b;
}

void terminate(int code) {
    exit(code);
}