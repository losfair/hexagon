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
            { OpCode::LoadInt(8) },
            { OpCode::InitLocal },
            { OpCode::LoadInt(0) }, // current (initial) value
            { OpCode::LoadInt(0) }, // slot id
            { OpCode::SetLocal },
            { OpCode::LoadInt(END) }, // end value (exclusive)
            { OpCode::LoadInt(1) }, // slot id
            { OpCode::SetLocal },
            { OpCode::LoadInt(0) }, // sum
            { OpCode::LoadInt(2) }, // slot id
            { OpCode::SetLocal },
            { OpCode::Branch(1) }
        ]),
        // bb 1
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(1) },
            { OpCode::GetLocal },
            { OpCode::LoadInt(0) },
            { OpCode::GetLocal },
            { OpCode::TestLt },
            { OpCode::Not },
            { OpCode::ConditionalBranch(3, 2) }
        ]),
        // bb 2
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(1) },
            { OpCode::LoadInt(0) },
            { OpCode::GetLocal },
            { OpCode::IntAdd },
            { OpCode::Dup },
            { OpCode::LoadInt(0) },
            { OpCode::SetLocal },
            { OpCode::LoadInt(2) },
            { OpCode::GetLocal },
            { OpCode::IntAdd },
            { OpCode::LoadInt(2) },
            { OpCode::SetLocal },
            { OpCode::Branch(1) }
        ]),
        // bb 3
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(2) },
            { OpCode::GetLocal },
            { OpCode::Return }
        ])
    ]));
    handle.create_static_object("sum", sum_fn);

    let blocks: Vec<BasicBlock> = vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(8) },
            { OpCode::InitLocal },
            { OpCode::LoadString("sum".to_string()) },
            { OpCode::GetStatic },
            { OpCode::LoadInt(0) },
            { OpCode::SetLocal },
            { OpCode::LoadNull },
            { OpCode::LoadInt(0) },
            { OpCode::GetLocal },
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
