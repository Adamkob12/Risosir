use core::ops::{Deref, DerefMut};
use spin::MutexGuard;

pub struct SpinLock<T>(spin::Mutex<T>);
