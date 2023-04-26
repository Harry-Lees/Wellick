#include <stdint.h>
#include <stdio.h>
#include <inttypes.h>

// A collection of C functions that are linked to Wellick
// and can be called from Wellick code. These functions act
// as the "builtins" of Wellick.

// The wellick "int" type is equivalent to a 32-bit unsigned integer
// So all integer functions use uint32_t in their parameters and return types

// Print a string to stdout
int32_t print(int64_t inp) {
    return printf("%" PRId64, inp);
}

int32_t println_32(int32_t inp) {
    return printf("%d\n", inp);
}

int32_t println_hex(int32_t inp) {
    return printf("0x%X\n", inp);
}

int64_t print_addr(int64_t inp) {
    return printf("0x%" PRIX64 "\n", inp);
}

// Print a string to stdout with a newline
int32_t println(int64_t inp) {
    return printf("%" PRId64 "\n", inp);
}

// Add two integers, return the result
int32_t iadd(int32_t a, int32_t b) {
    return a + b;
}

// Subtract two integers, return the result
int32_t isub(int32_t a, int32_t b) {
    return a - b;
}

// Multiply two integers, return the result
int32_t imul(int32_t a, int32_t b) {
    return a * b;
}

// Divide two integers, return the result
int32_t idiv(int32_t a, int32_t b) {
    return a / b;
}

// Check the equality of two integers, return 1 if true, 0 if false
int32_t ieq (int32_t a, int32_t b) {
    return a == b;
}

// Check if the first integer is less than the second, return 1 if true, 0 if false
int32_t ilt (int32_t a, int32_t b) {
    return a < b;
}

int32_t ilteq (int32_t a, int32_t b) {
    return a <= b;
}

// Check if the first integer is greater than the second, return 1 if true, 0 if false
int32_t igt (int32_t a, int32_t b) {
    return a > b;
}
