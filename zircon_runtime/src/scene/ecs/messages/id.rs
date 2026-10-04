use std::any::type_name;
use std::fmt;
use std::marker::PhantomData;

/// 可保留多帧的消息类型；自定义堆负载须报告实际保留字节，以维持 MessageRetention 的预算语义。
pub trait Message: 'static + Send + Sync {
    /// Returns the retention budget charged by this message instance.
    ///
    /// Messages that own heap data should override the inline-size default so
    /// the queue's byte ceiling remains an actual producer-visible limit.
    fn retained_byte_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

/// 单一 Messages<T> 通道递增分配的序号；用于游标与丢失计数，不是跨 World 身份。
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
