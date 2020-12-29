use crate::item::handle::Handle;
use crate::item::{ItemTrait};
use std::fmt::Debug;

pub trait Key<T>: ItemTrait + Debug {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct SkeletonKey {
    pub handle: Handle,
}

impl ItemTrait for SkeletonKey {
    fn name(&self) -> &str {
        "skeleton key"
    }

    fn display(&self) -> &str {
        "a rusted skeleton key"
    }

    fn description(&self) -> &str {
        "ok ok ok"
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    fn is_container(&self) -> bool {
        false
    }
}

impl Key<u64> for SkeletonKey {
    fn key(&self) -> u64 {
        1
    }
}
