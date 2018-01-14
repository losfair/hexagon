use super::program::{Program, NativeFunction};
use super::function::Function;
use super::basic_block::BasicBlock;
use super::opcode::OpCode;
use super::executor::Executor;

#[test]
fn test_ser_de() {
    fn test_feed(executor: &mut Executor) {
        executor.write_global(0, 42);
    }

    let mut program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::CallNative(0) },
                { OpCode::Return }
            ])
        ])
    ]);
    program.append_native_function(NativeFunction::new("test_feed", &test_feed));

    let info = program.dump();

    let program = Program::load(info, |name| {
        match name {
            "test_feed" => Some(NativeFunction::new("test_feed", &test_feed)),
            _ => None
        }
    }).unwrap();
    let mut executor = Executor::new();
    executor.eval_program(&program, 0);

    assert_eq!(executor.read_global(0), 42);
}
