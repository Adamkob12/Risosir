
pub struct SpinLock<T>(spin::Mutex<T>);
