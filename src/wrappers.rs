use std::marker::PhantomData;

mod vec;
pub use vec::Vec;

pub struct DefaultValue<T> {
    phantom: PhantomData<T>,
}

impl<T> DefaultValue<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

pub struct DefaultTrait<T> {
    phantom: PhantomData<T>,
}

impl<T: Default> DefaultTrait<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    pub fn get(&self) -> T {
        T::default()
    }
}

pub struct Option<T> {
    phantom: PhantomData<T>,
}

impl<T> Option<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
