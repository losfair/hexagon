use opcode::OpCode;
use errors;

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub(crate) opcodes: Vec<OpCode>
}

impl BasicBlock {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> BasicBlock {
        BasicBlock {
            opcodes: opcodes
        }
    }

    pub fn validate(&self) -> Result<(), errors::ValidateError> {
        let mut itr = self.opcodes.iter();
        let mut terminator_found: bool = false;
        let mut stack_depth: isize = 0;

        for op in &mut itr {
            let (n_pops, n_pushes) = op.get_stack_depth_change();

            stack_depth -= n_pops as isize;

            if stack_depth < 0 {
                return Err(errors::ValidateError::new("Invalid use of stack"));
            }

            stack_depth += n_pushes as isize;

            let terminated = match *op {
                OpCode::ConditionalBranch(_, _)
                    | OpCode::Branch(_)
                    | OpCode::Return => true,
                _ => false
            };
            if terminated {
                terminator_found = true;
                break;
            }
        }

        if stack_depth != 0 {
            return Err(errors::ValidateError::new(format!("Stack not empty at the end of basic block (Depth: {})", stack_depth)));
        }

        if itr.next().is_some() {
            return Err(errors::ValidateError::new("Invalid terminator found in basic block"));
        }

        if !terminator_found {
            return Err(errors::ValidateError::new("Terminator not found"));
        }

        Ok(())
    }
}
