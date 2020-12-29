use crate::item::handle::Handle;
use crate::item::Describe;
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct SkeletonKey {
    pub handle: Handle,
}

impl Describe for SkeletonKey {
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

#[derive(Debug, Clone)]
pub struct Codpiece(Handle);

impl Describe for Codpiece {
    fn name(&self) -> &str {
        "codpiece"
    }

    fn display(&self) -> &str {
        "A tattered old codpiece is here, mocking you."
    }

    fn description(&self) -> &str {
        "It's very ornate, but it's still very much a codpiece. You see no need for it, and yet \
        you simply can't resist the urge to put it on. You can't rationalize its power over you, and you \
        hang your head, ashamed."
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn is_container(&self) -> bool {
        false
    }
}

impl Key<u64> for Codpiece {
    fn key(&self) -> u64 {
        8
    }
}
