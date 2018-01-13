use super::page_table::PageTable;
use super::function::Function;
use super::opcode::OpCode;
use byteorder::{ReadBytesExt, NativeEndian};

pub struct Executor {
    page_table: PageTable
}

struct Local {
    regs: [u64; 16]
}

enum EvalControlMessage {
    Return,
    Redirect(usize)
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            page_table: PageTable::new()
        }
    }

    pub fn with_page_table(pt: PageTable) -> Executor {
        Executor {
            page_table: pt
        }
    }

    pub fn get_page_table(&self) -> &PageTable {
        &self.page_table
    }

    fn eval_partial(
        &mut self,
        local: &mut Local,
        f: &Function,
        block_id: usize
    ) -> EvalControlMessage {
        let blk = &f.basic_blocks[block_id];
        for op in blk.opcodes.iter() {
            match *op {
                OpCode::Return => {
                    return EvalControlMessage::Return;
                },
                OpCode::Branch(target) => {
                    return EvalControlMessage::Redirect(target);
                },
                OpCode::ConditionalBranch(a, b) => {
                    return EvalControlMessage::Redirect(if local.regs[0] != 0 {
                        a
                    } else {
                        b
                    });
                },
                OpCode::SIAdd(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 + local.regs[b] as i64) as u64;
                },
                OpCode::SISub(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 - local.regs[b] as i64) as u64;
                },
                OpCode::SIMul(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 * local.regs[b] as i64) as u64;
                },
                OpCode::SIDiv(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 / local.regs[b] as i64) as u64;
                },
                OpCode::SIMod(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 % local.regs[b] as i64) as u64;
                },
                OpCode::UIAdd(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 + local.regs[b] as u64) as u64;
                },
                OpCode::UISub(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 - local.regs[b] as u64) as u64;
                },
                OpCode::UIMul(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 * local.regs[b] as u64) as u64;
                },
                OpCode::UIDiv(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 / local.regs[b] as u64) as u64;
                },
                OpCode::UIMod(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 % local.regs[b] as u64) as u64;
                },
                OpCode::FAdd(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    local.regs[0] = unsafe {
                        ::std::mem::transmute::<f64, u64>(
                            (&left as &[u8]).read_f64::<NativeEndian>().unwrap() +
                            (&right as &[u8]).read_f64::<NativeEndian>().unwrap() 
                        )
                    };
                },
                OpCode::FSub(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    local.regs[0] = unsafe {
                        ::std::mem::transmute::<f64, u64>(
                            (&left as &[u8]).read_f64::<NativeEndian>().unwrap() -
                            (&right as &[u8]).read_f64::<NativeEndian>().unwrap() 
                        )
                    };
                },
                OpCode::FMul(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    local.regs[0] = unsafe {
                        ::std::mem::transmute::<f64, u64>(
                            (&left as &[u8]).read_f64::<NativeEndian>().unwrap() *
                            (&right as &[u8]).read_f64::<NativeEndian>().unwrap() 
                        )
                    };
                },
                OpCode::FDiv(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    local.regs[0] = unsafe {
                        ::std::mem::transmute::<f64, u64>(
                            (&left as &[u8]).read_f64::<NativeEndian>().unwrap() /
                            (&right as &[u8]).read_f64::<NativeEndian>().unwrap() 
                        )
                    };
                },
                OpCode::FMod(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    local.regs[0] = unsafe {
                        ::std::mem::transmute::<f64, u64>(
                            (&left as &[u8]).read_f64::<NativeEndian>().unwrap() %
                            (&right as &[u8]).read_f64::<NativeEndian>().unwrap() 
                        )
                    };
                },
                OpCode::Shl(a, b) => {
                    local.regs[0] = local.regs[a] << local.regs[b];
                },
                OpCode::Shr(a, b) => {
                    local.regs[0] = local.regs[a] >> local.regs[b];
                },
                OpCode::BitAnd(a, b) => {
                    local.regs[0] = local.regs[a] & local.regs[b];
                },
                OpCode::BitOr(a, b) => {
                    local.regs[0] = local.regs[a] | local.regs[b];
                },
                OpCode::Xor(a, b) => {
                    local.regs[0] = local.regs[a] ^ local.regs[b];
                },
                OpCode::LogicalNot(v) => {
                    local.regs[0] = if local.regs[v] != 0 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::BitNot(v) => {
                    local.regs[0] = !local.regs[v];
                },
                OpCode::SILt(a, b) => {
                    local.regs[0] = if (local.regs[a] as i64) < (local.regs[b] as i64) {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SILe(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 <= local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SIGe(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 >= local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SIGt(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 > local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UILt(a, b) => {
                    local.regs[0] = if (local.regs[a] as u64) < (local.regs[b] as u64) {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UILe(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 <= local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UIGe(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 >= local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UIGt(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 > local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FLt(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    let (left, right) = (
                        (&left as &[u8]).read_f64::<NativeEndian>().unwrap(),
                        (&right as &[u8]).read_f64::<NativeEndian>().unwrap()
                    );
                    local.regs[0] = if left < right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FLe(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    let (left, right) = (
                        (&left as &[u8]).read_f64::<NativeEndian>().unwrap(),
                        (&right as &[u8]).read_f64::<NativeEndian>().unwrap()
                    );
                    local.regs[0] = if left <= right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FGe(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    let (left, right) = (
                        (&left as &[u8]).read_f64::<NativeEndian>().unwrap(),
                        (&right as &[u8]).read_f64::<NativeEndian>().unwrap()
                    );
                    local.regs[0] = if left >= right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FGt(a, b) => {
                    let (left, right) = unsafe {
                        (
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[a]),
                            ::std::mem::transmute::<u64, [u8; 8]>(local.regs[b])
                        )
                    };
                    let (left, right) = (
                        (&left as &[u8]).read_f64::<NativeEndian>().unwrap(),
                        (&right as &[u8]).read_f64::<NativeEndian>().unwrap()
                    );
                    local.regs[0] = if left > right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::Eq(a, b) => {
                    local.regs[0] = if local.regs[a] == local.regs[b] {
                        1
                    } else {
                        0
                    };
                },
                OpCode::Ne(a, b) => {
                    local.regs[0] = if local.regs[a] != local.regs[b] {
                        1
                    } else {
                        0
                    };
                },
                OpCode::Load8(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.read_u8(addr).unwrap() as u64;
                },
                OpCode::Load16(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.read_u16(addr).unwrap() as u64;
                },
                OpCode::Load32(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.read_u32(addr).unwrap() as u64;
                },
                OpCode::Load64(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.read_u64(addr).unwrap() as u64;
                },
                OpCode::Store8(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.write_u8(addr, (local.regs[src] & 0xff) as u8);
                },
                OpCode::Store16(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.write_u16(addr, (local.regs[src] & 0xffff) as u16);
                },
                OpCode::Store32(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.write_u32(addr, (local.regs[src] & 0xffffffff) as u32);
                },
                OpCode::Store64(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.write_u64(addr, local.regs[src]);
                }
            }
        }
        panic!("Terminator not found");
    }

    pub fn eval_function(&mut self, f: &Function) {
        let mut local = Local {
            regs: [0u64; 16]
        };
        let mut block_id: usize = 0;

        loop {
            match self.eval_partial(&mut local, f, block_id) {
                EvalControlMessage::Return => {
                    break;
                },
                EvalControlMessage::Redirect(target) => {
                    block_id = target;
                }
            }
        }
    }
}
