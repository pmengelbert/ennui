use crate::attribute::{Attribute, Quality};
use crate::describe::{Describe, Description};
use crate::error::{CmdErr, EnnuiError};
use crate::hook::{Grabber, Hook};
use crate::text::{message::MessageFormat, Color};

use std::fmt::Debug;
use std::mem::take;

#[derive(Default, Debug)]
pub struct List<T, U> {
    inner: Vec<T>,
    info: Description,
    attr: Vec<U>,
}

impl<T: Default, U> List<T, U> {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
            attr: vec![],
        }
    }

    pub fn set_info(&mut self, i: Description) {
        self.info = i;
    }

    pub fn set_attr_list(&mut self, v: Vec<U>) {
        self.attr = v;
    }

    pub fn into_inner(mut self) -> Vec<T> {
        take(&mut self.inner)
    }
}

impl<T: Default, U> IntoIterator for List<T, U> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T: Default + Send + Sync + Debug, U: Send + Sync + Debug> Describe for List<T, U> {
    fn handle(&self) -> Hook {
        self.info.handle()
    }

    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        self.info.description()
    }
}

impl<T: Default + Send + Sync + Debug> Attribute<Quality> for List<T, Quality> {
    fn is(&self, a: Quality) -> bool {
        self.attr.contains(&a)
    }

    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.attr.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let pos = self.attr.iter().position(|u| *u == q);
        if let Some(pos) = pos {
            self.attr.remove(pos);
        }
    }
}

pub trait ListTrait: Describe + Debug {
    type Item: Describe + Default + Debug;

    fn get_item(&self, handle: Grabber) -> Option<&Self::Item>;
    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Self::Item>;
    fn get_item_owned(&mut self, handle: Grabber) -> Result<Self::Item, EnnuiError>;
    fn insert_item(&mut self, item: Self::Item) -> Result<(), Self::Item>;
    fn list(&self) -> Vec<&Self::Item>;
    fn display_items(&self) -> String;

    fn transfer(
        &mut self,
        other: &mut ListTrait<Item = Self::Item>,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let item = self.get_item_owned(handle.into())?;

        let name = item.name();
        if other.insert_item(item).is_err() {
            return Err(EnnuiError::Fatal("COULD NOT TRANSFER ITEM".into()));
        };
        Ok(name)
    }
}

impl<T: Describe + Default + Debug, U: Send + Sync + Debug> ListTrait for List<T, U> {
    type Item = T;

    fn get_item(&self, handle: Grabber) -> Option<&T> {
        self.inner
            .iter()
            .filter(|i| i.handle() == handle.handle)
            .nth(handle.index)
    }

    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut T> {
        self.inner
            .iter_mut()
            .filter(|i| i.handle() == handle.handle)
            .nth(handle.index)
    }

    fn get_item_owned(&mut self, handle: Grabber) -> Result<T, EnnuiError> {
        let index = self
            .inner
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if item.handle() == handle.handle {
                    Some(i)
                } else {
                    None
                }
            })
            .nth(handle.index);

        let index = match index {
            Some(i) => i,
            None => return Err(EnnuiError::Simple(CmdErr::ItemNotFound)),
        };

        Ok(self.inner.remove(index))
    }

    fn display_items(&self) -> String {
        let mut ret = String::new();
        let lst = self.list();
        if lst.len() > 0 {
            ret.push('\n');
        }
        ret.push_str(
            &self
                .list()
                .iter()
                .map(|i| i.display().color(Color::Green))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        ret
    }

    fn insert_item(&mut self, item: T) -> Result<(), T> {
        self.inner.push(item);
        Ok(())
    }

    fn list(&self) -> Vec<&T> {
        self.inner.iter().collect()
    }
}
