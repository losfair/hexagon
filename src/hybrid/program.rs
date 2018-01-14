use std::fmt::{Debug, Formatter};
use super::function::Function;
use super::executor::Executor;

#[derive(Debug)]
pub struct Program<'a> {
    pub(super) functions: Vec<Function>,
    pub(super) native_functions: Vec<NativeFunction<'a>>
}

pub struct NativeFunction<'a> {
    fn_impl: &'a Fn(&mut Executor)
}

impl<'a> Debug for NativeFunction<'a> {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "NativeFunction")
    }
}

impl<'a> NativeFunction<'a> {
    pub fn new(f: &'a Fn(&mut Executor)) -> NativeFunction<'a> {
        NativeFunction {
            fn_impl: f
        }
    }

    pub fn invoke(&self, executor: &mut Executor) {
        (self.fn_impl)(executor);
    }
}

impl<'a> Program<'a> {
    pub fn from_functions(fns: Vec<Function>) -> Program<'a> {
        Program {
            functions: fns,
            native_functions: Vec::new()
        }
    }

    pub fn append_native_function(&mut self, f: NativeFunction<'a>) -> usize {
        self.native_functions.push(f);
        self.native_functions.len() - 1
    }
}
