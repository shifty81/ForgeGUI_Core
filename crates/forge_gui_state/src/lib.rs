//! Lightweight observable state primitives.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Observable<T> {
    value: T,
    revision: u64,
    last_origin: Option<String>,
}
impl<T> Observable<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            revision: 0,
            last_origin: None,
        }
    }
    pub fn get(&self) -> &T {
        &self.value
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn last_origin(&self) -> Option<&str> {
        self.last_origin.as_deref()
    }
    pub fn set(&mut self, value: T, origin: impl Into<String>) {
        self.value = value;
        self.revision = self.revision.wrapping_add(1);
        self.last_origin = Some(origin.into());
    }
    pub fn update(&mut self, origin: impl Into<String>, f: impl FnOnce(&mut T)) {
        f(&mut self.value);
        self.revision = self.revision.wrapping_add(1);
        self.last_origin = Some(origin.into());
    }
}
pub struct Derived<T> {
    compute: Box<dyn Fn() -> T>,
}
impl<T> Derived<T> {
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        Self {
            compute: Box::new(f),
        }
    }
    pub fn get(&self) -> T {
        (self.compute)()
    }
}
