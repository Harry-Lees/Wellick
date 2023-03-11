#include <stdint.h>

uint32_t iadd(uint32_t a, uint16_t b) {
    return a + b;
}

uint32_t isub(uint32_t a, uint16_t b) {
    return a - b;
}

uint32_t imul(uint32_t a, uint16_t b) {
    return a * b;
}

uint32_t idiv(uint32_t a, uint16_t b) {
    return a / b;
}

uint32_t ieq (uint32_t a, uint16_t b) {
    return a == b;
}

uint32_t ilt (uint32_t a, uint16_t b) {
    return a < b;
}

uint32_t igt (uint32_t a, uint16_t b) {
    return a > b;
}