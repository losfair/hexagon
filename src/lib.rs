#![feature(nll)]

extern crate smallvec;
extern crate byteorder;

pub mod hybrid;

pub mod basic_block;
pub mod call_stack;
pub mod errors;
pub mod executor;
pub mod function_optimizer;
pub mod function;
pub mod object_info;
pub mod object_pool;
pub mod object;
pub mod opcode;
pub mod primitive;
pub mod static_root;
pub mod value;

#[cfg(test)]
mod executor_test;
