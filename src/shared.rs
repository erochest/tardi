use std::cell::RefCell;
use std::rc::Rc;

/// Shared type
pub type Shared<T> = Rc<RefCell<T>>;

/// Helper function to create a SharedValue
pub fn shared<V>(value: V) -> Shared<V> {
    Rc::new(RefCell::new(value))
}
