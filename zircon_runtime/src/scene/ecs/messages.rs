use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
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
        write!(
            formatter,
            "message<{}>#{}",
            type_name::<T>()
                .rsplit("::")
                .next()
                .unwrap_or(type_name::<T>()),
            self.id
        )
    }
}

struct MessageInstance<T>
where
    T: Message,
{
    id: MessageId<T>,
    message: T,
}

pub struct Messages<T>
where
    T: Message,
{
    messages: Vec<MessageInstance<T>>,
    next_id: usize,
    // Cursor reset marker for explicit retention boundaries such as clear_messages.
    generation: u64,
}

impl<T> Default for Messages<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 0,
            generation: 0,
        }
    }
}

impl<T> Messages<T>
where
    T: Message,
{
    pub fn write(&mut self, message: T) -> MessageId<T> {
        let id = MessageId::new(self.next_id);
        self.next_id += 1;
        self.messages.push(MessageInstance { id, message });
        id
    }

    pub fn iter(&self) -> MessageReadIter<'_, T> {
        MessageReadIter::new(self.messages.iter())
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.generation = self.generation.saturating_add(1);
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

pub struct MessageCursor<T>
where
    T: Message,
{
    cursor: usize,
    generation: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for MessageCursor<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            cursor: 0,
            generation: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> MessageCursor<T>
where
    T: Message,
{
    pub fn read<'a>(&mut self, messages: Option<&'a Messages<T>>) -> MessageReadIter<'a, T> {
        let Some(messages) = messages else {
            self.cursor = 0;
            self.generation = 0;
            return MessageReadIter::empty();
        };
        let start = if self.generation == messages.generation() {
            self.cursor.min(messages.messages.len())
        } else {
            0
        };
        self.cursor = messages.messages.len();
        self.generation = messages.generation();
        MessageReadIter::new(messages.messages[start..].iter())
    }

    pub fn unread_count(&self, messages: Option<&Messages<T>>) -> usize {
        messages
            .map(|messages| {
                if self.generation == messages.generation() {
                    messages
                        .messages
                        .len()
                        .saturating_sub(self.cursor.min(messages.messages.len()))
                } else {
                    messages.messages.len()
                }
            })
            .unwrap_or_default()
    }

    pub fn clear(&mut self, messages: Option<&Messages<T>>) {
        if let Some(messages) = messages {
            self.cursor = messages.len();
            self.generation = messages.generation();
        } else {
            self.cursor = 0;
            self.generation = 0;
        }
    }
}

pub struct MessageReadIter<'a, T>
where
    T: Message,
{
    inner: Option<std::slice::Iter<'a, MessageInstance<T>>>,
}

impl<'a, T> MessageReadIter<'a, T>
where
    T: Message,
{
    fn new(inner: std::slice::Iter<'a, MessageInstance<T>>) -> Self {
        Self { inner: Some(inner) }
    }

    fn empty() -> Self {
        Self { inner: None }
    }
}

impl<'a, T> Iterator for MessageReadIter<'a, T>
where
    T: Message,
{
    type Item = (MessageId<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.as_mut()?.next()?;
        Some((next.id, &next.message))
    }
}

#[derive(Default)]
pub struct MessageStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    type_names: HashMap<TypeId, &'static str>,
}

impl MessageStore {
    pub fn messages<T>(&self) -> Option<&Messages<T>>
    where
        T: Message,
    {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|store| store.downcast_ref::<Messages<T>>())
    }

    pub fn messages_mut<T>(&mut self) -> &mut Messages<T>
    where
        T: Message,
    {
        let type_id = TypeId::of::<T>();
        self.type_names.entry(type_id).or_insert(type_name::<T>());
        self.stores
            .entry(type_id)
            .or_insert_with(|| Box::<Messages<T>>::default())
            .downcast_mut::<Messages<T>>()
            .expect("message store type id must match message queue type")
    }

    pub fn write<T>(&mut self, message: T) -> MessageId<T>
    where
        T: Message,
    {
        self.messages_mut::<T>().write(message)
    }

    pub fn clear<T>(&mut self)
    where
        T: Message,
    {
        self.messages_mut::<T>().clear();
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let mut names = self.type_names.values().copied().collect::<Vec<_>>();
        names.sort_unstable();
        names
    }
}

impl fmt::Debug for MessageStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageStore")
            .field("registered_type_names", &self.registered_type_names())
            .finish()
    }
}

impl Clone for MessageStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for MessageStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
