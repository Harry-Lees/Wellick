use super::ast;
use super::variables;

use cranelift::prelude::AbiParam;
use cranelift::prelude::InstBuilder;
use cranelift::prelude::MemFlags;
use cranelift_codegen::ir::{entities::Value, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;
use std::process;

/// Module to translate AST into Cranelift IR constructs.
pub struct FunctionTranslator<'a, 'b: 'a> {
    pub(crate) builder: FunctionBuilder<'b>,
    pub(crate) variables: HashMap<String, variables::Variable>,
    pub(crate) module: &'a mut ObjectModule,
}

impl<'a, 'b> FunctionTranslator<'a, 'b> {
    pub fn new(
        builder: FunctionBuilder<'b>,
        variables: HashMap<String, variables::Variable>,
        module: &'b mut ObjectModule,
    ) -> Self {
        Self {
            builder,
            variables,
            module,
        }
    }

    pub fn translate_if(
        &mut self,
        condition: &ast::Expression,
        if_body: &Vec<ast::Stmt>,
        else_body: &Option<Vec<ast::Stmt>>,
    ) -> Value {
        let condition_value = self.translate_expr(condition);

        let then_block = self.builder.create_block();
        let merge_block = self.builder.create_block();
        self.builder.ins().brz(condition_value, then_block, &[]);
        self.builder.ins().jump(merge_block, &[]);
        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);

        let mut then_return: Option<Value> = None;
        for expr in if_body {
            then_return = Some(self.translate_stmt(expr));
        }
        if then_return.is_none() {
            panic!("If statement has no body");
        }

        self.builder.switch_to_block(merge_block);
        then_return.unwrap()
    }

    pub fn translate_stmt(&mut self, stmt: &ast::Stmt) -> Value {
        match stmt {
            ast::Stmt::Assign(expr) => self.translate_assign(expr),
            ast::Stmt::If(condition, body, else_body) => {
                self.translate_if(condition, body, else_body)
            }
            ast::Stmt::Return(expr) => self.translate_return(expr),
            ast::Stmt::Call(expr) => self.translate_call(expr),
            ast::Stmt::ReAssign(expr) => self.translate_reassign(expr),
        }
    }

    pub fn translate_expr(&mut self, expr: &ast::Expression) -> Value {
        match expr {
            ast::Expression::Call(val) => self.translate_call(val),
            ast::Expression::Literal(val) => match val {
                ast::Value::F32(value) => self.builder.ins().f32const(*value),
                ast::Value::I32(value) => self.builder.ins().iconst(types::I32, *value as i64),
            },
            ast::Expression::Identifier(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                self.builder.ins().stack_load(var.var_type, var.loc, 0)
            }
            // Address-Of a value, returns a pointer pointing to the stack slot
            // of the variable.
            ast::Expression::AddressOf(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                let stack_addr = self.builder.ins().stack_addr(
                    self.module.target_config().pointer_type(),
                    var.loc,
                    0,
                );
                stack_addr
            }
            // Dereference a pointer and return the value at that address.
            ast::Expression::DeRef(value) => {
                let var = self
                    .variables
                    .get(value)
                    .expect("No variable with that name could be found");

                // Load the pointer from the stack slot.
                let stack_ptr = self.builder.ins().stack_load(
                    // The pointer itself will always be the target's pointer size.
                    self.module.target_config().pointer_type(),
                    var.loc,
                    0,
                );

                println!("Dereferencing variable of type {:?}", var.var_type);
                // Load the value pointed to by "stack_ptr"
                let val = self
                    .builder
                    .ins()
                    .load(var.var_type, MemFlags::new(), stack_ptr, 0);
                val
            }
        }
    }

    fn translate_call(&mut self, expr: &ast::Call) -> Value {
        let mut sig = self.module.make_signature();
        let mut arg_values = Vec::new();
        for arg in &expr.args {
            let func_arg = self.translate_expr(arg);
            arg_values.push(func_arg);
            // let arg_type = self.builder.ins().data_flow_graph().value_type(func_arg);
            let arg_type = self.builder.func.dfg.value_type(func_arg);
            sig.params.push(AbiParam::new(arg_type));
        }
        sig.returns.push(AbiParam::new(types::I32));
        let callee = self
            .module
            .declare_function(&expr.func, Linkage::Import, &sig)
            .expect("Unable to declare function");
        let local_callee = self.module.declare_func_in_func(callee, self.builder.func);
        let call = self.builder.ins().call(local_callee, &arg_values);
        self.builder.inst_results(call)[0]
    }

    fn translate_return(&mut self, expr: &ast::Expression) -> Value {
        let value = self.translate_expr(expr);
        self.builder.ins().return_(&[value]);
        value
    }

    fn translate_reassign(&mut self, expr: &ast::Local) -> Value {
        let name = &expr.target.ident;
        let value = self.translate_expr(&expr.value);

        let var = self.variables.get(name);
        if var.is_none() {
            println!("Cannot find value `{name}` in this scope");
            process::exit(1);
        }

        if !var.unwrap().mutable {
            println!("Cannot mutate immutable variable {name}");
            process::exit(1);
        }

        self.builder.def_var(var.unwrap().reference, value);

        value
    }

    fn translate_assign(&mut self, expr: &ast::Assignment) -> Value {
        let name = &expr.target.ident;
        let value = self.translate_expr(&expr.value);
        let var = self
            .variables
            .get(name)
            .expect(format!("No variable named {}", name).as_str());

        let value_type = self.builder.func.dfg.value_type(value);
        if value_type != var.var_type {
            println!(
                "Cannot convert type from {} to {}",
                value_type, var.var_type
            );
            process::exit(1);
        };

        self.builder.ins().stack_store(value, var.loc, 0);
        value
    }
}
