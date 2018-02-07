use std::collections::{HashSet, BTreeSet};
use basic_block::BasicBlock;
use opcode::{OpCode, RtOpCode};
use object::Object;
use object_pool::ObjectPool;
use value::Value;

pub struct FunctionOptimizer<'a> {
    basic_blocks: &'a mut Vec<BasicBlock>,
    rt_handles: &'a mut Vec<usize>,
    pool: &'a mut ObjectPool
}

impl<'a> FunctionOptimizer<'a> {
    pub fn new(basic_blocks: &'a mut Vec<BasicBlock>, rt_handles: &'a mut Vec<usize>, pool: &'a mut ObjectPool) -> FunctionOptimizer<'a> {
        FunctionOptimizer {
            basic_blocks: basic_blocks,
            rt_handles: rt_handles,
            pool: pool
        }
    }

    pub fn optimize(&mut self) {
        // Optimizes out (LoadString, GetStatic) sequences if possible.
        // This requires const string information.
        self.optimize_const_static_load();

        // Const string information will be lost after this;
        self.optimize_string_load();

        // Run optimizations on each basic block
        for bb in self.basic_blocks.iter_mut() {
            bb.transform_const_get_fields(self.rt_handles, self.pool);
            bb.transform_const_calls();
            bb.remove_nops();

            // Some stack pushes / pops will be removed after this
            // so this should be the last optimization
            bb.rebuild_stack_patterns();
        }

        self.simplify_cfg();
    }

    pub fn simplify_cfg(&mut self) {
        if self.basic_blocks.len() == 0 {
            return;
        }

        let mut n_basic_blocks: usize = self.basic_blocks.len();

        let mut out_edges: Vec<HashSet<usize>> = vec! [ HashSet::new(); n_basic_blocks ];
        let mut in_edges: Vec<HashSet<usize>> = vec! [ HashSet::new(); n_basic_blocks ];
        for i in 0..n_basic_blocks {
            let (a, b) = self.basic_blocks[i].branch_targets();
            if let Some(v) = a {
                out_edges[i].insert(v);
                in_edges[v].insert(i);
            }
            if let Some(v) = b {
                out_edges[i].insert(v);
                in_edges[v].insert(i);
            }
        }

        for i in 0..n_basic_blocks {
            if out_edges[i].len() == 1 {
                let j = *out_edges[i].iter().nth(0).unwrap();
                if in_edges[j].len() == 1 {
                    if *in_edges[j].iter().nth(0).unwrap() == i {
                        debug!("[simplify_cfg] Found unique connection: {} <-> {}", i, j);
                        out_edges.swap(i, j);
                        out_edges[j].clear();
                        in_edges[j].clear();
                        let v = ::std::mem::replace(
                            &mut self.basic_blocks[j],
                            BasicBlock::from_opcodes(Vec::new())
                        );
                        self.basic_blocks[i].join(v);
                    }
                }
            }
        }

        let mut dfs_stack: Vec<usize> = Vec::new();
        let mut dfs_visited: Vec<bool> = vec![ false; n_basic_blocks ];

        dfs_visited[0] = true;
        dfs_stack.push(0);

        while !dfs_stack.is_empty() {
            let current = dfs_stack.pop().unwrap();

            for other in &out_edges[current] {
                if !dfs_visited[*other] {
                    dfs_visited[*other] = true;
                    dfs_stack.push(*other);
                }
            }
        }

        // collect unused blocks
        {
            let unused_blocks: BTreeSet<usize> = (0..self.basic_blocks.len()).filter(|i| !dfs_visited[*i]).collect();
            let mut tail = n_basic_blocks - 1;
            let mut remap_list: Vec<(usize, usize)> = Vec::new(); // (to, from)
            for id in &unused_blocks {
                while tail > *id {
                    if unused_blocks.contains(&tail) {
                        tail -= 1;
                    } else {
                        break;
                    }
                }

                // Implies tail > 0
                if tail <= *id {
                    break;
                }

                // Now `id` is the first unused block and `tail`
                // is the last used block
                // Let's exchange them
                remap_list.push((*id, tail));
                self.basic_blocks.swap(*id, tail);
                tail -= 1;
            }
            while self.basic_blocks.len() > tail + 1 {
                self.basic_blocks.pop().unwrap();
            }
            for (to, from) in remap_list {
                for bb in self.basic_blocks.iter_mut() {
                    let replaced = bb.try_replace_branch_targets(to, from);
                    if replaced {
                        debug!("[simplify_cfg] Branch target replaced: {} -> {}", from, to);
                    }
                }
            }
            n_basic_blocks = self.basic_blocks.len();
        }
    }

    pub fn optimize_string_load(&mut self) {
        for bb in self.basic_blocks.iter_mut() {
            for op in bb.opcodes.iter_mut() {
                if let OpCode::LoadString(ref s) = *op {
                    let s = s.clone();
                    let obj = self.pool.allocate(Box::new(s));
                    self.rt_handles.push(obj);
                    *op = OpCode::Rt(RtOpCode::LoadObject(obj));
                }
            }
        }
    }

    pub fn optimize_const_static_load(&mut self) {
        for bb in self.basic_blocks.iter_mut() {
            let mut new_opcodes: Vec<OpCode> = Vec::new();
            let mut patterns: Vec<Option<String>> = vec![None; bb.opcodes.len()];

            for i in 0..bb.opcodes.len() - 1 {
                match (&bb.opcodes[i], &bb.opcodes[i + 1]) {
                    (&OpCode::LoadString(ref s), &OpCode::GetStatic) => {
                        patterns[i] = Some(s.clone());
                    },
                    _ => {}
                }
            }

            let mut remove_count: usize = 0;

            for i in 0..bb.opcodes.len() {
                if let Some(ref name) = patterns[i] {
                    if let Some(val) = self.pool.get_static_object(name) {
                        let op = match *val {
                            Value::Bool(v) => OpCode::LoadBool(v),
                            Value::Float(v) => OpCode::LoadFloat(v),
                            Value::Int(v) => OpCode::LoadInt(v),
                            Value::Null => OpCode::LoadNull,
                            Value::Object(id) => {
                                self.rt_handles.push(id);
                                OpCode::Rt(RtOpCode::LoadObject(id))
                            }
                        };
                        new_opcodes.push(op);
                        // the next opcode will be the original `GetStatic`, which is not needed any more.
                        remove_count = 1;
                        continue;
                    }
                }

                if remove_count > 0 {
                    remove_count -= 1;
                } else {
                    new_opcodes.push(bb.opcodes[i].clone());
                }
            }

            bb.opcodes = new_opcodes;
            match bb.validate(true) {
                Ok(_) => {},
                Err(e) => panic!(e.to_str().to_string())
            }
        }
    }
}
