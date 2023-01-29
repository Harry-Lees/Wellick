extern crate cranelift;

use cranelift::{
    codegen::{settings, Context},
    prelude::{AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, Type},
};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_native::builder as host_isa_builder;
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::{fs::File, io::Write};

fn main() {
    let isa_builder = host_isa_builder().expect("host machine is not a supported target");
    let isa = isa_builder
        .finish(settings::Flags::new(settings::builder()))
        .unwrap();

    let builder = ObjectBuilder::new(
        isa,
        "test".to_string(),
        cranelift_module::default_libcall_names(),
    )
    .unwrap();
    let mut module = ObjectModule::new(builder);

    // Define an integer type for use in the program.
    let int = module.target_config().pointer_type();

    let mut data_context = DataContext::new();
    let mut codegen_context = Context::new();
    let mut function_builder_context = FunctionBuilderContext::new();

    // Create static data in __text block
    data_context.define("hello world\0".as_bytes().to_vec().into_boxed_slice());

    let data_id = module
        .declare_data("data", Linkage::Local, true, false)
        .unwrap();

    module
        .define_data(data_id, &data_context)
        .map_err(|e| e.to_string())
        .unwrap();

    data_context.clear();

    codegen_context
        .func
        .signature
        .returns
        .push(AbiParam::new(int));

    // Define a function
    let mut function_builder =
        FunctionBuilder::new(&mut codegen_context.func, &mut function_builder_context);
    let entry_block = function_builder.create_block();
    function_builder.switch_to_block(entry_block);

    let value = function_builder
        .ins()
        .iconst(Type::int_with_byte_size(8).unwrap(), 0);
    function_builder.ins().return_(&[value]);

    function_builder.seal_block(entry_block);
    function_builder.finalize();

    // Declare a function, has to be done before the function can be
    // Called or defined.
    let function_id = module
        .declare_function("main", Linkage::Export, &codegen_context.func.signature)
        .expect("Failed to create function_id");

    // Define the function on the module
    module
        .define_function(function_id, &mut codegen_context)
        .expect("Failed to define module function");

    module.clear_context(&mut codegen_context);

    // Finish
    let product = module.finish();
    let code = product.emit().unwrap();

    let mut file = File::create("a.out").expect("Failed to create file");
    file.write_all(&code).unwrap();
}
