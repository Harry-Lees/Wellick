use super::ast;
use super::ast::{FloatType, IntegerType};
use cranelift::prelude::StackSlotData;
use cranelift_codegen::ir::StackSlot;
use cranelift_codegen::ir::{types, Block};
use cranelift_frontend::FunctionBuilder;
use cranelift_frontend::Variable as cranelift_Variable;
use std::collections::HashMap;
use std::str::FromStr;

/// Helper function to convert the EmptyType AST node to
/// a valid Cranelift IR type.
pub(crate) fn to_cranelift_type(t: &ast::EmptyType) -> types::Type {
    match t {
        ast::EmptyType::Float(FloatType::F32) => types::F32,
        ast::EmptyType::Float(FloatType::F64) => types::F64,
        ast::EmptyType::Integer(IntegerType::I32) => types::I32,
        ast::EmptyType::Integer(IntegerType::I64) => types::I64,
        // TODO: This only works for platforms with a 64bit integer size.
        ast::EmptyType::Integer(IntegerType::PointerSize) => types::I64,
        // TODO: This also only works for platforms with a 64bit pointer size.
        ast::EmptyType::Pointer(_) => types::I64,
    }
}

#[derive(Debug, Clone)]
pub struct StackVar {
    pub name: String,
    pub ty: ast::EmptyType,
    pub base: StackSlot,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct RegVar {
    pub name: String,
    pub ty: ast::EmptyType,
    pub base: cranelift_Variable,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub enum Variable {
    Stack(StackVar),
    Register(RegVar),
}

impl StackVar {
    fn new(name: String, ty: ast::EmptyType, base: StackSlot, mutable: bool) -> Self {
        Self {
            name,
            ty,
            mutable,
            base,
        }
    }

    fn alloc(
        name: String,
        ty: ast::EmptyType,
        mutable: bool,
        builder: &mut FunctionBuilder,
        index: &mut usize,
        variables: &mut HashMap<String, Variable>,
    ) -> Self {
        if variables.contains_key(&name) {
            panic!("Cannot re-declare variable {}", name);
        }

        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            cranelift::prelude::StackSlotKind::ExplicitSlot,
            to_cranelift_type(&ty).bytes(),
        ));

        let var = Self::new(name.clone(), ty, stack_slot, mutable);
        variables.insert(name, Variable::Stack(var.clone()));
        *index += 1;
        var
    }
}

impl RegVar {
    fn new(name: String, ty: ast::EmptyType, base: cranelift_Variable, mutable: bool) -> Self {
        Self {
            name,
            ty,
            mutable,
            base,
        }
    }

    fn alloc(
        name: String,
        ty: ast::EmptyType,
        mutable: bool,
        builder: &mut FunctionBuilder,
        index: &mut usize,
        variables: &mut HashMap<String, Variable>,
    ) -> Self {
        if variables.contains_key(&name) {
            panic!("Cannot re-declare variable {}", name);
        }

        let var_ref = cranelift_Variable::from_u32(*index as u32);
        builder.declare_var(var_ref, to_cranelift_type(&ty));

        let var = Self::new(name.clone(), ty, var_ref, mutable);
        variables.insert(name, Variable::Register(var.clone()));
        *index += 1;
        var
    }
}

impl Variable {
    pub fn ty(&self) -> ast::EmptyType {
        match self {
            Variable::Stack(var) => var.ty.clone(),
            Variable::Register(var) => var.ty.clone(),
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
        let var = RegVar::alloc(
            arg.name.clone(),
            arg.t.clone(),
            false,
            builder,
            &mut index,
            &mut variables,
        );
        builder.def_var(var.base, val);
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
            StackVar::alloc(
                assignment.target.ident.clone(),
                assignment.var_type.clone(),
                assignment.mutable,
                builder,
                index,
                variables,
            );
        }
        ast::Stmt::If(ref _condition, ref if_body, ref else_body) => {
            for stmt in if_body {
                declare_variables_in_stmt(stmt, builder, index, variables);
            }
        }
        _ => {}
    }
}
