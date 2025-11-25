use std::cell::RefCell;
use std::rc::{Rc, Weak};

// EffectFn is a boxed callable
type EffectFn = Rc<dyn Fn()>;

pub struct Signal<T> {
    value: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<Vec<Weak<RefCell<EffectFn>>>>>,
}

impl<T: Clone> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        let mut subs = self.subscribers.borrow_mut();
        subs.retain(|weak_cb| {
            if let Some(cb) = weak_cb.upgrade() {
                cb.borrow()();
                true
            } else {
                false
            }
        });
    }

    pub(crate) fn subscribe(&self, effect: &Rc<RefCell<EffectFn>>) {
        self.subscribers.borrow_mut().push(Rc::downgrade(effect));
    }
}

pub struct Computed<T> {
    value: Rc<RefCell<T>>,
}

impl<T: Clone + 'static> Computed<T> {
    pub fn new<U: Clone + 'static, F>(source: &Signal<U>, compute: F) -> Self
    where
        F: Fn(U) -> T + 'static,
    {
        let initial = compute(source.get());
        let value = Rc::new(RefCell::new(initial));

        let value_clone = value.clone();
        let compute_clone = Rc::new(compute);
        // 🟢 Wrap closure in Rc<dyn Fn()>
        let closure: Rc<dyn Fn()> = Rc::new(move || {
            let new_val = compute_clone(source.get());
            *value_clone.borrow_mut() = new_val;
        });

        // 🟢 Now RefCell holds Rc<dyn Fn()>
        let effect_fn = Rc::new(RefCell::new(closure));

        source.subscribe(&effect_fn);

        Self { value }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }
}
