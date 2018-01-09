#![feature(nll)]

extern crate smallvec;

pub mod basic_block;
pub mod call_stack;
pub mod errors;
pub mod executor;
pub mod function;
pub mod object_info;
pub mod object_pool;
pub mod object;
pub mod opcode;
pub mod primitive;
pub mod static_root;

#[cfg(test)]
pub mod executor_test;
