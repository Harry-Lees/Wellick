/// A collection of builtin functions linked to the Wellick compiler to be used at runtime.

/// Integer add function to add two i32 integers.
pub extern "C" fn iadd(x: i32, y: i32) -> i32 {
    return x + y;
}

/// Floating point add function to add two 32-bit floats.
pub extern "C" fn fadd(x: f32, y: f32) -> f32 {
    return x + y;
}
