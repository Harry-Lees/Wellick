use super::ast;
use cranelift::prelude::StackSlotData;
use cranelift_codegen::ir::StackSlot;
use cranelift_codegen::ir::{types, Block};
use cranelift_frontend::FunctionBuilder;
use cranelift_frontend::Variable as cranelift_Variable;
use std::collections::HashMap;

/// Helper function to convert the EmptyType AST node to
/// a valid Cranelift IR type.
pub(crate) fn to_cranelift_type(t: &ast::EmptyType) -> types::Type {
    match t {
        ast::EmptyType::Float => types::F64,
        ast::EmptyType::Integer => types::I32,
        ast::EmptyType::Pointer => types::I64,
    }
}

#[derive(Debug, Clone)]
pub enum VarLoc {
    StackSlot(StackSlot),
    Register,
}

/// A Variable declaration, holds the name, type, cranelift ref,
/// and the mutability of the variable.
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: types::Type,
    pub reference: cranelift_Variable,
    pub loc: VarLoc,
    pub mutable: bool,
}

impl Variable {
    fn new(
        name: String,
        var_type: types::Type,
        reference: cranelift_Variable,
        loc: VarLoc,
        mutable: bool,
    ) -> Self {
        Self {
            name,
            var_type,
            mutable,
            reference,
            loc,
        }
    }
}

pub fn declare_variables(
    node: &ast::FnDecl,
    builder: &mut FunctionBuilder,
    entry_block: Block,
) -> HashMap<String, Variable> {
    let args = &node.args;
    let mut variables = HashMap::<String, Variable>::new();
    let mut index: usize = 0;

    let params = builder.block_params(entry_block).to_vec();
    for (i, arg) in args.iter().enumerate() {
        let val = params[i];
        let var = alloc(
            arg.name.clone(),
            to_cranelift_type(&arg.t),
            false,
            builder,
            &mut index,
            &mut variables,
        );
        builder.def_var(var.reference, val);
    }

    for expr in &node.body {
        declare_variables_in_stmt(expr, builder, &mut index, &mut variables);
    }
    variables
}

/// Recursively descend through the AST, translating all implicit
/// variable declarations.
fn declare_variables_in_stmt(
    expr: &ast::Stmt,
    builder: &mut FunctionBuilder,
    index: &mut usize,
    variables: &mut HashMap<String, Variable>,
) {
    match expr {
        ast::Stmt::Assign(ref assignment) => {
            if assignment.var_type.is_some() {
                alloc(
                    assignment.target.ident.clone(),
                    to_cranelift_type(&assignment.var_type.clone().unwrap()),
                    assignment.mutable,
                    builder,
                    index,
                    variables,
                );
            }
        }
        ast::Stmt::If(ref _condition, ref if_body, ref else_body) => {
            for stmt in if_body {
                declare_variables_in_stmt(stmt, builder, index, variables);
            }

            if else_body.is_some() {
                todo!("else block not currently implemented");
            }
        }
        _ => {}
    }
}

/// Allocate a stack slot for a variable.
/// Allocating in this way allows grabbing a pointer
/// to a local variable.
fn alloc(
    name: String,
    var_type: types::Type,
    mutable: bool,
    builder: &mut FunctionBuilder,
    index: &mut usize,
    variables: &mut HashMap<String, Variable>,
) -> Variable {
    if variables.contains_key(&name) {
        panic!("Cannot re-declare variable {}", name);
    }

    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
        cranelift::prelude::StackSlotKind::ExplicitSlot,
        var_type.bytes(),
    ));

    let var = Variable::new(
        name.clone(),
        var_type,
        cranelift_Variable::from_u32(*index as u32),
        VarLoc::StackSlot(stack_slot),
        mutable,
    );
    variables.insert(name, var.clone());
    *index += 1;
    var
}

/// Allocate a register for a variable.
/// Allocating in this way disallows getting local pointers
/// to variables.
fn reg_alloc(
    name: String,
    var_type: types::Type,
    mutable: bool,
    builder: &mut FunctionBuilder,
    index: &mut usize,
    variables: &mut HashMap<String, Variable>,
) -> Variable {
    if variables.contains_key(&name) {
        panic!("Cannot re-declare variable {}", name);
    }
    let var = Variable::new(
        name.clone(),
        var_type,
        cranelift_Variable::from_u32(*index as u32),
        VarLoc::Register,
        mutable,
    );
    builder.declare_var(var.reference, var.var_type);
    variables.insert(name, var.clone());
    *index += 1;
    var
}
