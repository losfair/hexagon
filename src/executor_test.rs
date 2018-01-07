use executor::Executor;
use opcode::OpCode;
use basic_block::BasicBlock;
use function::Function;
use object::Object;
use primitive::Int;

#[test]
fn test_executor() {
    let executor = Executor::new();
    let mut handle = executor.handle_mut();

    let blocks: Vec<BasicBlock> = vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(42) },
            { OpCode::LoadString("output".to_string()) },
            { OpCode::SetStatic },
            { OpCode::LoadNull },
            { OpCode::Return }
        ])
    ];
    handle.create_static_object("entry", Box::new(Function::from_basic_blocks(blocks)));
    match handle.run_callable("entry") {
        Ok(_) => {},
        Err(e) => panic!(e.unwrap().to_string())
    }

    let output_slot = handle.get_static_object_ref("output").unwrap();
    let result = output_slot.as_any().downcast_ref::<Int>().unwrap().to_i64();

    assert_eq!(result, 42);
}
