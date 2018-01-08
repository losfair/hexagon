use object::Object;
use object_info::{ObjectInfo, ObjectHandle, TypedObjectHandle};
use static_root::StaticRoot;
use call_stack::CallStack;

pub struct ObjectPool {
    objects: Vec<Option<ObjectInfo>>,
    object_idx_pool: Vec<usize>
}

impl ObjectPool {
    pub fn new() -> ObjectPool {
        ObjectPool {
            objects: vec![
                Some(ObjectInfo::new(Box::new(StaticRoot::new())))
            ],
            object_idx_pool: vec![]
        }
    }

    pub fn allocate(&mut self, inner: Box<Object>) -> usize {
        let pool = &mut self.object_idx_pool;
        let id = if let Some(id) = pool.pop() {
            id
        } else {
            let objects = &mut self.objects;
            objects.push(None);
            objects.len() - 1
        };
        self.objects[id] = Some(ObjectInfo::new(inner));
        id
    }

    fn deallocate(&mut self, id: usize) {
        let objects = &mut self.objects;
        let pool = &mut self.object_idx_pool;

        assert!(objects[id].is_some());

        objects[id] = None;
        pool.push(id);
    }

    pub fn get<'a>(&self, id: usize) -> ObjectHandle<'a> {
        self.objects[id].as_ref().unwrap().handle()
    }

    pub fn get_typed<'a, T: 'static>(&self, id: usize) -> Option<TypedObjectHandle<'a, T>> {
        TypedObjectHandle::downcast_from(self.get(id))
    }

    pub fn get_static_root<'a>(&self) -> TypedObjectHandle<'a, StaticRoot> {
        self.get_typed(0).unwrap()
    }

    pub fn collect(&mut self, stack: &CallStack) {
        let mut visited: Vec<bool> = vec![false; self.objects.len()];

        let mut dfs: Vec<usize> = Vec::new();
        dfs.push(0); // static root

        for id in stack.collect_objects() {
            visited[id] = true;
            dfs.push(id);
        }

        while !dfs.is_empty() {
            let id = dfs.pop().unwrap();

            if visited[id] {
                continue;
            }
            visited[id] = true;

            let obj = &self.objects[id].as_ref().unwrap();
            for child in obj.as_object().get_children() {
                dfs.push(child);
            }
        }

        for i in 0..visited.len() {
            if self.objects[i].is_some() && !visited[i] {
                self.objects[i].as_mut().unwrap().gc_notify();
                self.deallocate(i);
            }
        }
    }
}

impl Drop for ObjectPool {
    fn drop(&mut self) {
        for obj in &mut self.objects {
            if let Some(ref mut obj) = *obj {
                obj.gc_notify();
            }
        }
    }
}
