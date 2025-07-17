use std::sync::{Arc, RwLock, RwLockReadGuard};

pub mod node;
pub type WireDataContainer<T> = Arc<RwLock<T>>;
pub type WireDataReference<'a, T> = RwLockReadGuard<'a, T>;
