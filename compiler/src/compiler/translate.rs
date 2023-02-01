use super::ast;
use cranelift::prelude::{AbiParam, InstBuilder};
use cranelift_codegen::ir::{entities::Value, types, Block};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{Linkage, Module};
use cranelift_object::{object::Object, ObjectModule};
use std::collections::HashMap;

/// Module to translate AST into Cranelift IR constructs.

/// Helper function to convert the EmptyType AST node to
/// a valid Cranelift IR type.
pub(crate) fn to_cranelift_type(t: &ast::EmptyType) -> types::Type {
    match t {
        ast::EmptyType::Float => types::F64,
        ast::EmptyType::Integer => types::I64,
    }
}

/// Helper function to convert the Value AST node to
/// a valid Cranelift IR type.
pub(crate) fn value_type(t: &ast::Value) -> types::Type {
    match t {
        ast::Value::Float(_) => types::F64,
        ast::Value::Integer(_) => types::I64,
    }
}

pub(crate) fn declare_variables(
    builder: &mut FunctionBuilder,
    args: &Vec<ast::FnArg>,
    stmts: &Vec<ast::Expression>,
    entry_block: Block,
) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, arg) in args.iter().enumerate() {
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(
            to_cranelift_type(&arg.t),
            builder,
            &mut variables,
            &mut index,
            &arg.name,
        );
        builder.def_var(var, val);
    }

    for expr in stmts {
        declare_variables_in_stmt(builder, &mut variables, &mut index, expr);
    }

    variables
}

/// Declare a single variable declaration.
pub(crate) fn declare_variable(
    var_type: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    dbg!(var_type);
    let var = Variable::from_u32(*index as u32);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, var_type);
        *index += 1;
    }
    var
}

/// Recursively descend through the AST, translating all implicit
/// variable declarations.
pub(crate) fn declare_variables_in_stmt(
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    expr: &ast::Expression,
) -> Variable {
    dbg!(expr);
    match expr {
        ast::Expression::Assign(ref assignment) => declare_variable(
            value_type(&assignment.value),
            builder,
            variables,
            index,
            assignment.target.ident.as_ref(),
        ),
        ast::Expression::Call(_) => {
            todo!();
        }
        ast::Expression::If(_, _, _) => {
            todo!();
        }
    }
}

pub struct FunctionTranslator<'a> {
    pub(crate) builder: FunctionBuilder<'a>,
    pub(crate) variables: HashMap<String, Variable>,
    pub(crate) module: &'a mut ObjectModule,
}

impl<'a> FunctionTranslator<'a> {
    pub fn new(
        builder: FunctionBuilder<'a>,
        variables: HashMap<String, Variable>,
        module: &'a mut ObjectModule,
    ) -> Self {
        Self {
            builder,
            variables,
            module,
        }
    }

    pub fn translate_expr(&mut self, expr: &ast::Expression) -> Value {
        match expr {
            ast::Expression::Assign(val) => self.translate_assign(val),
            ast::Expression::If(_, _, _) => todo!(),
            ast::Expression::Call(_) => todo!(),
        }
    }

    fn translate_assign(&mut self, expr: &ast::Assignment) -> Value {
        let name = &expr.target.ident;
        let var = self
            .variables
            .get(name)
            .expect(format!("No variable named {}", name).as_str());
        let value = match &expr.value {
            ast::Value::Float(val) => self.builder.ins().f64const(*val as f64),
            ast::Value::Integer(val) => self.builder.ins().iconst(types::I64, *val as i64),
        };
        self.builder.def_var(*var, value);
        value
    }
}
