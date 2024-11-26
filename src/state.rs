use std::sync::{Arc, Mutex, Once};

#[derive(Debug, Default)]

pub struct Store {
    pub verbose_enabled: bool,
}

pub struct State {
    store: Arc<Mutex<Store>>,
}

impl State {
    pub fn global() -> &'static State {
        // We use Once to ensure the initialization happens exactly once
        static ONCE: Once = Once::new();
        static mut INSTANCE: Option<State> = None;

        unsafe {
            ONCE.call_once(|| {
                // Initialize the singleton instance
                let state = State {
                    store: Arc::new(Mutex::new(Store::default())),
                };
                INSTANCE = Some(state);
            });
            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<Store> {
        self.store.lock().unwrap()
    }
}
