use std::sync::Arc;

pub trait Arced {
    fn arced(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
}

impl<T> Arced for T {}
