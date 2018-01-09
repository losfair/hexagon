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

    const END: i64 = 100000;

    let sum_fn = Box::new(Function::from_basic_blocks(vec! [
        // bb 0
        BasicBlock::from_opcodes(vec! [
            { OpCode::InitLocal(8) },
            { OpCode::LoadInt(0) }, // current (initial) value
            { OpCode::SetLocal(0) },
            { OpCode::LoadInt(END) }, // end value (exclusive)
            { OpCode::SetLocal(1) },
            { OpCode::LoadInt(0) }, // sum
            { OpCode::SetLocal(2) },
            { OpCode::Branch(1) }
        ]),
        // bb 1
        BasicBlock::from_opcodes(vec! [
            { OpCode::GetLocal(1) },
            { OpCode::GetLocal(0) },
            { OpCode::TestLt },
            { OpCode::Not },
            { OpCode::ConditionalBranch(3, 2) }
        ]),
        // bb 2
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(1) },
            { OpCode::GetLocal(0) },
            { OpCode::IntAdd },
            { OpCode::Dup },
            { OpCode::SetLocal(0) },
            { OpCode::GetLocal(2) },
            { OpCode::IntAdd },
            { OpCode::SetLocal(2) },
            { OpCode::Branch(1) }
        ]),
        // bb 3
        BasicBlock::from_opcodes(vec! [
            { OpCode::GetLocal(2) },
            { OpCode::Return }
        ])
    ]));
    handle.create_static_object("sum", sum_fn);

    let blocks: Vec<BasicBlock> = vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::InitLocal(8) },
            { OpCode::LoadString("sum".to_string()) },
            { OpCode::GetStatic },
            { OpCode::SetLocal(0) },
            { OpCode::LoadNull },
            { OpCode::GetLocal(0) },
            { OpCode::Call(0) },
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

    handle.gc();

    let output_slot = handle.get_static_object_ref("output").unwrap();
    let result = output_slot.as_any().downcast_ref::<Int>().unwrap().to_i64();

    assert_eq!(result, (1 + END) * END / 2);
}
