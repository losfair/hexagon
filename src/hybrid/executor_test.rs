use super::executor::Executor;
use super::function::Function;
use super::basic_block::BasicBlock;
use super::opcode::OpCode;
use super::page_table::PageTable;

#[test]
fn test_sum() {
    const END: u64 = 100000;

    let mut pt = PageTable::new();
    pt.virtual_alloc(0x08000000);
    pt.write_u64(0x08000000, 0); // exclusive
    pt.write_u64(0x08000008, END); // inclusive

    let sum_fn = Function::from_basic_blocks(vec![
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 0x08000000) },
            { OpCode::Load64(1, 0) },
            { OpCode::UIConst64(0, 0x08000008) },
            { OpCode::Load64(2, 0) },
            { OpCode::Branch(1) }
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIGe(1, 2) },
            { OpCode::ConditionalBranch(3, 2)}
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 1) },
            { OpCode::UIAdd(0, 1) },
            { OpCode::Mov(1, 0) },
            { OpCode::UIAdd(1, 3) },
            { OpCode::Mov(3, 0) },
            { OpCode::Branch(1) }
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 0x08000016) },
            { OpCode::Store64(3, 0) },
            { OpCode::Return }
        ])
    ]);

    let mut executor = Executor::with_page_table(pt.clone());
    executor.eval_function(&sum_fn);

    let result = pt.read_u64(0x08000016).unwrap();
    assert_eq!(result, (1 + END) * END / 2);
}
