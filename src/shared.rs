use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// Shared type
pub type Shared<T> = Rc<RefCell<T>>;

/// Helper function to create a SharedValue
pub fn shared<V>(value: V) -> Shared<V> {
    Rc::new(RefCell::new(value))
}

pub fn unshare_clone<V: Clone>(shared_value: Shared<V>) -> V {
    V::clone(Rc::unwrap_or_clone(shared_value).borrow().deref())
}
