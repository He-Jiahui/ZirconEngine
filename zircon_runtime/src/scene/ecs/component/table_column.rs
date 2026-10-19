use std::alloc::Layout;
use std::any::TypeId;
use std::fmt;
use std::mem;
use std::ptr::NonNull;

use crate::scene::ecs::storage::StoredComponent;

/// Type metadata and move/drop functions for one dense archetype column.
///
/// The registry creates this once for every Rust `Table` component. The table
/// storage uses the erased callbacks only after it has checked `type_id`, so
/// hot queries never depend on `Any` values as their persistent body owner.
#[derive(Clone)]
pub(crate) struct TableColumnLayout {
    type_id: TypeId,
    type_name: &'static str,
    layout: Layout,
    write_box: unsafe fn(StoredComponent, *mut u8),
    take_box: unsafe fn(*mut u8) -> StoredComponent,
    drop_value: unsafe fn(*mut u8),
}

impl TableColumnLayout {
    pub(crate) fn of<T>() -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            write_box: write_box::<T>,
            take_box: take_box::<T>,
            drop_value: drop_value::<T>,
        }
    }

    pub(crate) fn accepts(&self, value: &StoredComponent) -> bool {
        value.as_ref().type_id() == self.type_id
    }

    pub(crate) fn matches<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.type_id == TypeId::of::<T>()
    }

    pub(crate) fn layout(&self) -> Layout {
        self.layout
    }

    pub(crate) fn is_zero_sized(&self) -> bool {
        self.layout.size() == 0
    }

    pub(crate) fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub(crate) unsafe fn write_box(&self, value: StoredComponent, destination: *mut u8) {
        // SAFETY: callers validate the component TypeId and reserve an aligned,
        // uninitialized slot for this layout before invoking the callback.
        unsafe { (self.write_box)(value, destination) };
    }

    pub(crate) unsafe fn take_box(&self, source: *mut u8) -> StoredComponent {
        // SAFETY: callers pass one initialized slot owned by this column.
        unsafe { (self.take_box)(source) }
    }

    pub(crate) unsafe fn drop_value(&self, value: *mut u8) {
        // SAFETY: callers pass one initialized slot owned by this column.
        unsafe { (self.drop_value)(value) };
    }
}

impl fmt::Debug for TableColumnLayout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TableColumnLayout")
            .field("type_name", &self.type_name)
            .field("layout", &self.layout)
            .finish()
    }
}

impl PartialEq for TableColumnLayout {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.layout == other.layout
    }
}

impl Eq for TableColumnLayout {}

unsafe fn write_box<T>(value: StoredComponent, destination: *mut u8)
where
    T: Send + Sync + 'static,
{
    let value = value
        .downcast::<T>()
        .expect("validated dense archetype column must receive its registered Rust type");
    if mem::size_of::<T>() == 0 {
        mem::forget(*value);
        return;
    }
    // SAFETY: `destination` is an aligned uninitialized slot for `T`.
    unsafe { destination.cast::<T>().write(*value) };
}

unsafe fn take_box<T>(source: *mut u8) -> StoredComponent
where
    T: Send + Sync + 'static,
{
    if mem::size_of::<T>() == 0 {
        // SAFETY: the caller owns one initialized logical row created from a
        // real T value. This zero-byte move still needs a non-null pointer
        // aligned for T; the column's erased ZST pointer need not be aligned.
        // The returned Box takes the row's sole drop obligation. The caller
        // must reinitialize or remove the row before it can be taken again.
        return Box::new(unsafe { NonNull::<T>::dangling().as_ptr().read() });
    }
    // SAFETY: `source` is an initialized, aligned slot for `T`; this moves it
    // out and leaves the slot uninitialized for replacement or removal.
    Box::new(unsafe { source.cast::<T>().read() })
}

unsafe fn drop_value<T>(value: *mut u8)
where
    T: Send + Sync + 'static,
{
    let value = if mem::size_of::<T>() == 0 {
        NonNull::<T>::dangling().as_ptr()
    } else {
        value.cast::<T>()
    };
    // SAFETY: callers retain exactly one initialized logical instance per row.
    unsafe { std::ptr::drop_in_place(value) };
}
