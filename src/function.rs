use std::any::Any;
use object::Object;
use object_pool::ObjectPool;
use basic_block::BasicBlock;
use executor::ExecutorImpl;
use errors;
use function_optimizer::FunctionOptimizer;
use value::Value;

pub enum Function {
    Virtual(VirtualFunction),
    Native(NativeFunction)
}

pub struct VirtualFunction {
    basic_blocks: Vec<BasicBlock>,
    rt_handles: Vec<usize>,
    should_optimize: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VirtualFunctionInfo {
    pub basic_blocks: Vec<BasicBlock>
}

pub type NativeFunction = Box<Fn(&mut ExecutorImpl) -> Value>;

impl Object for Function {
    fn initialize(&mut self, pool: &mut ObjectPool) {
        if let Function::Virtual(ref mut f) = *self {
            if f.should_optimize {
                f.optimize(pool);
            }
        }
    }

    fn get_children(&self) -> Vec<usize> {
        match *self {
            Function::Virtual(ref f) => f.rt_handles.clone(),
            Function::Native(_) => Vec::new()
        }
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn call(&self, executor: &mut ExecutorImpl) -> Value {
        match *self {
            Function::Virtual(ref vf) => {
                executor.eval_basic_blocks(vf.basic_blocks.as_slice(), 0)
            },
            Function::Native(ref nf) => {
                nf(executor)
            }
        }
    }
}

impl Function {
    pub fn from_basic_blocks(blocks: Vec<BasicBlock>) -> Function {
        let vf = VirtualFunction {
            basic_blocks: blocks,
            rt_handles: Vec::new(),
            should_optimize: false
        };

        vf.validate().unwrap_or_else(|e| {
            panic!(errors::VMError::from(e))
        });

        Function::Virtual(vf)
    }

    pub fn enable_optimization(&mut self) {
        if let Function::Virtual(ref mut f) = *self {
            f.should_optimize = true;
        } else {
            panic!("Optimization is not supported on the current function type");
        }
    }

    pub fn from_native(nf: NativeFunction) -> Function {
        Function::Native(nf)
    }

    pub fn to_virtual_info(&self) -> Option<VirtualFunctionInfo> {
        match *self {
            Function::Virtual(ref vf) => Some(VirtualFunctionInfo {
                basic_blocks: vf.basic_blocks.clone()
            }),
            Function::Native(_) => None
        }
    }

    pub fn from_virtual_info(vinfo: VirtualFunctionInfo) -> Self {
        Function::from_basic_blocks(vinfo.basic_blocks)
    }
}

impl VirtualFunction {
    fn optimize(&mut self, pool: &mut ObjectPool) {
        let mut optimizer = FunctionOptimizer::new(&mut self.basic_blocks, &mut self.rt_handles, pool);
        optimizer.optimize();
    }

    pub fn validate(&self) -> Result<(), errors::ValidateError> {
        self.validate_basic_blocks()?;
        self.validate_branch_targets()?;
        Ok(())
    }

    pub fn validate_basic_blocks(&self) -> Result<(), errors::ValidateError> {
        for bb in &self.basic_blocks {
            bb.validate(false)?;
        }

        Ok(())
    }

    pub fn validate_branch_targets(&self) -> Result<(), errors::ValidateError> {
        let blocks = &self.basic_blocks;

        for bb in blocks {
            let mut found_error: bool = false;

            let (fst, snd) = bb.branch_targets();
            if let Some(fst) = fst {
                if fst >= blocks.len() {
                    found_error = true;
                }
            }
            if let Some(snd) = snd {
                if snd >= blocks.len() {
                    found_error = true;
                }
            }

            if found_error {
                return Err(errors::ValidateError::new("Invalid branch target(s)"));
            }
        }

        Ok(())
    }
}
