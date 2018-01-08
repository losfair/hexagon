use object::Object;
use object_info::{ObjectInfo, ObjectHandle, TypedObjectHandle};
use static_root::StaticRoot;

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

    pub fn deallocate(&mut self, id: usize) {
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
