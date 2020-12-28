use crate::item::{ItemTrait, ItemKind};
use crate::item::handle::Handle;
use crate::item::ItemKind::Container;

pub trait Key<T> : ItemTrait {
    fn key(&self) -> T;
}

#[derive(Clone)]
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

    fn kind(&self) -> ItemKind {
        Container
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl Key<u64> for SkeletonKey {
    fn key(&self) -> u64 {
        1
    }
}