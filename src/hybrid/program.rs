use std::fmt::{Debug, Formatter};
use super::function::Function;
use super::executor::Executor;

#[derive(Debug)]
pub struct Program<'a> {
    pub functions: Vec<Function>,
    pub(super) native_functions: Vec<NativeFunction<'a>>
}

#[derive(Clone, Debug)]
pub struct ProgramInfo<'a> {
    pub functions: Vec<Function>,
    pub native_functions: Vec<&'a str>
}

pub struct NativeFunction<'a> {
    name: String,
    fn_impl: &'a Fn(&Executor)
}

impl<'a> Debug for NativeFunction<'a> {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "NativeFunction")
    }
}

impl<'a> NativeFunction<'a> {
    pub fn new<T: ToString>(name: T, f: &'a Fn(&Executor)) -> NativeFunction<'a> {
        NativeFunction {
            fn_impl: f,
            name: name.to_string()
        }
    }

    pub fn invoke(&self, executor: &Executor) {
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

    pub fn resolve_native_functions<T, F>(
        &mut self,
        function_list: &[T],
        resolver: F
    ) -> bool
        where
            F: Fn(&str) -> Option<NativeFunction<'a>>,
            T: AsRef<str>
    {
        let mut nf_list: Vec<NativeFunction<'a>> = Vec::new();

        for name in function_list {
            let nf = if let Some(v) = resolver(name.as_ref()) {
                v
            } else {
                return false;
            };
            nf_list.push(nf);
        }

        self.native_functions = nf_list;
        true
    }

    pub fn dump(&self) -> ProgramInfo {
        ProgramInfo {
            functions: self.functions.clone(),
            native_functions: self.native_functions.iter().map(|v| v.name.as_str()).collect()
        }
    }

    pub fn load<F>(info: ProgramInfo, resolver: F) -> Option<Program<'a>>
        where F: Fn(&str) -> Option<NativeFunction<'a>>
    {
        let mut program = Self::from_functions(info.functions);
        if program.resolve_native_functions(info.native_functions.as_slice(), resolver) == false {
            return None;
        }

        Some(program)
    }
}
