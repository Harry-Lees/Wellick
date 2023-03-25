///! Defines pointer which is used to generate CLIF IR for pointer operations.
///! Adapted from https://github.com/bjorn3/rustc_codegen_cranelift/blob/master/src/pointer.rs
use cranelift::prelude::{types, InstBuilder, Value};
use cranelift_codegen::ir::{immediates::Offset32, StackSlot};
use cranelift_frontend::FunctionBuilder;

/// A pointer pointing to a stack slot.
pub(crate) struct Pointer {
    base: StackSlot,
    offset: Offset32,
}

impl Pointer {
    pub(crate) fn new(stack_slot: StackSlot) -> Self {
        Self {
            base: stack_slot,
            offset: Offset32::new(0),
        }
    }

    /// Load the address of the pointer.
    pub(crate) fn get_addr(self, builder: &mut FunctionBuilder) -> Value {
        builder.ins().stack_addr(types::I64, self.base, self.offset)
    }

    /// Load the value pointed to by this pointer.
    pub(crate) fn load(self, type_: types::Type, builder: &mut FunctionBuilder) -> Value {
        builder.ins().stack_load(type_, self.base, self.offset)
    }

    /// Store a given value at the pointer address.
    pub(crate) fn store(self, value: Value, builder: &mut FunctionBuilder) {
        builder.ins().stack_store(value, self.base, self.offset);
    }
}
