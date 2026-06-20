use std::any::type_name;
use std::fmt;
use std::marker::PhantomData;

pub trait Message: 'static + Send + Sync {}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageId<T>
where
    T: Message,
{
    id: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T> MessageId<T>
where
    T: Message,
{
    pub const fn new(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub const fn id(self) -> usize {
        self.id
    }
}

impl<T> Clone for MessageId<T>
where
    T: Message,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MessageId<T> where T: Message {}

impl<T> fmt::Debug for MessageId<T>
where
    T: Message,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message_type_name = type_name::<T>();
        let message_type_label = match message_type_name.rsplit("::").next() {
            Some(label) => label,
            None => message_type_name,
        };

        write!(formatter, "message<{}>#{}", message_type_label, self.id)
    }
}
