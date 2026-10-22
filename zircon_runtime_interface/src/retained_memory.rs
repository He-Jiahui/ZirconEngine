//! Versioned retained-allocation bounds for the admitted Rust/Global model.
//!
//! Bounds cover requested owned allocation layouts, not allocator bookkeeping or RSS.
//! Source/package provenance is an external actual-build/release admission premise;
//! the local gate observes only the compiled representation and model availability.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::mem::{align_of, align_of_val, needs_drop, size_of, size_of_val};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize};
use std::sync::{Arc, Mutex};

use thiserror::Error;

mod availability {
    include!(concat!(env!("OUT_DIR"), "/retained_std_model_v1.rs"));
}

pub const RETAINED_PROVIDER_VERSION_V1: u32 = 1;
/// Provider traversal policy, independent of the admitted allocation model.
pub const MAX_RETAINED_DEPTH_V1: usize = 64;
pub const MAX_RETAINED_VISITS_V1: usize = 1_048_576;
pub const RETAINED_MODEL_ID_V1: &str = availability::ZR_RETAINED_STD_MODEL_ID_V1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetainedBytesV1(u64);

impl RetainedBytesV1 {
    pub const ZERO: Self = Self(0);
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
    pub fn checked_add(self, other: Self) -> Result<Self, RetentionContractError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(RetentionContractError::Overflow)
    }
    pub fn checked_mul(self, count: u64) -> Result<Self, RetentionContractError> {
        self.0
            .checked_mul(count)
            .map(Self)
            .ok_or(RetentionContractError::Overflow)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetainedAccountingModelV1 {
    _private: (),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetainedBoundV1 {
    pub model: RetainedAccountingModelV1,
    pub max_bytes: RetainedBytesV1,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RetentionContractError {
    #[error("retained allocation bound overflow")]
    Overflow,
    #[error("retention provider nesting exceeds {limit}")]
    DepthLimit { limit: usize },
    #[error("retention provider traversal exceeds {limit} values")]
    WorkLimit { limit: usize },
    #[error("unsupported retained allocation model: {reason}")]
    UnsupportedConfiguration { reason: &'static str },
    #[error("retention provider contract failed: {reason}")]
    Provider { reason: &'static str },
    #[error("retention allocation could not be reserved")]
    Allocation,
    #[error("retention budget {budget} refused {requested} bytes (limit {limit})")]
    Budget {
        budget: &'static str,
        requested: u64,
        limit: u64,
    },
    #[error("retention preparation owner or revision changed")]
    Stale,
}

pub type RetentionConfigurationError = RetentionContractError;

/// A recursive provider must count every owned allocation reachable through this owner.
/// The containing allocation bills inline storage; this method adds owned children only.
/// Erasure must retain this provider. Shared mutable children need an enforced lifetime
/// maximum or a quota-controlled owner; observing their current length is insufficient.
pub trait RetainedOwnerV1 {
    fn retained_bound_v1(
        &self,
        meter: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError>;
}

/// Shared references cannot grow the transitive bound behind the history owner.
///
/// # Safety
/// No mutation through an alias may create uncharged retained allocation. Interior
/// mutable implementations must enforce a finite lifetime cap or use an owning ledger.
pub unsafe trait ImmutableRetainedOwnerV1: RetainedOwnerV1 {}

/// A concrete value has no transitive owned allocation, including through shared state.
///
/// # Safety
/// Every valid value and every mutation of it must retain no allocation of its own.
pub unsafe trait FixedRetainedOwnerV1: ImmutableRetainedOwnerV1 {}

pub fn verified_std_model_v1() -> Result<RetainedAccountingModelV1, RetentionConfigurationError> {
    if !availability::ZR_RETAINED_STD_MODEL_AVAILABLE_V1 {
        return Err(RetentionContractError::UnsupportedConfiguration {
            reason: availability::ZR_RETAINED_STD_MODEL_REASON_V1,
        });
    }
    // This discriminates only the audited closed set of source representations.
    // Same-layout unknown source/version/patch provenance is NOT detected here.
    let tree = size_of::<BTreeMap<String, serde_json::Value>>();
    if tree > 32
        || size_of::<serde_json::Map<String, serde_json::Value>>() != tree
        || align_of::<serde_json::Map<String, serde_json::Value>>()
            != align_of::<BTreeMap<String, serde_json::Value>>()
        || needs_drop::<serde_json::Number>()
    {
        return Err(RetentionContractError::UnsupportedConfiguration {
            reason:
                "normal serde_json representation is not the audited tree/ordinary-number profile",
        });
    }
    Ok(RetainedAccountingModelV1 { _private: () })
}

pub struct RetainedBoundBuilderV1 {
    model: RetainedAccountingModelV1,
    bytes: RetainedBytesV1,
    limit: RetainedBytesV1,
    depth: usize,
    visits: usize,
}

impl RetainedBoundBuilderV1 {
    pub fn new(model: RetainedAccountingModelV1) -> Self {
        Self::with_limit(model, RetainedBytesV1::new(u64::MAX))
    }
    pub fn with_limit(model: RetainedAccountingModelV1, limit: RetainedBytesV1) -> Self {
        Self {
            model,
            bytes: RetainedBytesV1::ZERO,
            limit,
            depth: 0,
            visits: 0,
        }
    }
    pub fn items(&mut self, count: usize) -> Result<(), RetentionContractError> {
        let visits = self
            .visits
            .checked_add(count)
            .ok_or(RetentionContractError::Overflow)?;
        if visits > MAX_RETAINED_VISITS_V1 {
            return Err(RetentionContractError::WorkLimit {
                limit: MAX_RETAINED_VISITS_V1,
            });
        }
        self.visits = visits;
        Ok(())
    }
    /// Checks before descending; an error always restores the caller's depth.
    pub fn nest(
        &mut self,
        visit: impl FnOnce(&mut Self) -> Result<(), RetentionContractError>,
    ) -> Result<(), RetentionContractError> {
        if self.depth >= MAX_RETAINED_DEPTH_V1 {
            return Err(RetentionContractError::DepthLimit {
                limit: MAX_RETAINED_DEPTH_V1,
            });
        }
        self.items(1)?;
        self.depth += 1;
        let result = visit(self);
        self.depth -= 1;
        result
    }
    pub const fn model(&self) -> RetainedAccountingModelV1 {
        self.model
    }
    pub const fn bytes(&self) -> RetainedBytesV1 {
        self.bytes
    }
    pub const fn finish(self) -> RetainedBoundV1 {
        RetainedBoundV1 {
            model: self.model,
            max_bytes: self.bytes,
        }
    }
    pub fn add(&mut self, bytes: u64) -> Result<(), RetentionContractError> {
        let next = self.bytes.checked_add(RetainedBytesV1::new(bytes))?;
        if next > self.limit {
            return Err(RetentionContractError::Budget {
                budget: "provider traversal",
                requested: next.get(),
                limit: self.limit.get(),
            });
        }
        self.bytes = next;
        Ok(())
    }
    pub fn array<T>(&mut self, capacity: usize) -> Result<(), RetentionContractError> {
        let bytes = capacity
            .checked_mul(size_of::<T>())
            .ok_or(RetentionContractError::Overflow)?;
        if bytes > isize::MAX as usize {
            return Err(RetentionContractError::Overflow);
        }
        self.add(u64::try_from(bytes).map_err(|_| RetentionContractError::Overflow)?)
    }
    pub fn layout(&mut self, bytes: usize, alignment: usize) -> Result<(), RetentionContractError> {
        if !alignment.is_power_of_two() || bytes > isize::MAX as usize {
            return Err(RetentionContractError::Overflow);
        }
        self.add(u64::try_from(bytes).map_err(|_| RetentionContractError::Overflow)?)
    }
    pub fn owned<T: RetainedOwnerV1 + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), RetentionContractError> {
        self.layout(size_of_val(value), align_of_val(value))?;
        value.retained_bound_v1(self)
    }
    pub fn btree<K, V>(&mut self, len: usize) -> Result<(), RetentionContractError> {
        self.add(
            btree_node_upper_v1::<K, V>()?
                .checked_mul(
                    u64::try_from(len.max(1)).map_err(|_| RetentionContractError::Overflow)?,
                )
                .ok_or(RetentionContractError::Overflow)?,
        )
    }
    pub fn btree_construction<K, V>(&mut self, len: usize) -> Result<(), RetentionContractError> {
        let nodes = len
            .checked_mul(2)
            .and_then(|n| n.checked_add(3))
            .ok_or(RetentionContractError::Overflow)?;
        self.add(
            btree_node_upper_v1::<K, V>()?
                .checked_mul(u64::try_from(nodes).map_err(|_| RetentionContractError::Overflow)?)
                .ok_or(RetentionContractError::Overflow)?,
        )
    }
    pub fn arc_payload<T: RetainedOwnerV1 + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), RetentionContractError> {
        let alignment = align_of_val(value);
        let bytes = align_up_v1(16, alignment)?
            .checked_add(size_of_val(value))
            .ok_or(RetentionContractError::Overflow)?;
        let bytes = align_up_v1(bytes, alignment.max(8))?;
        self.layout(bytes, alignment.max(8))?;
        value.retained_bound_v1(self)
    }
}

pub fn retained_bound_v1<T: RetainedOwnerV1 + ?Sized>(
    value: &T,
    model: RetainedAccountingModelV1,
) -> Result<RetainedBoundV1, RetentionContractError> {
    let mut meter = RetainedBoundBuilderV1::new(model);
    meter.owned(value)?;
    Ok(meter.finish())
}

pub fn align_up_v1(bytes: usize, alignment: usize) -> Result<usize, RetentionContractError> {
    if !alignment.is_power_of_two() {
        return Err(RetentionContractError::Overflow);
    }
    bytes
        .checked_add(alignment - 1)
        .map(|n| n & !(alignment - 1))
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(RetentionContractError::Overflow)
}

pub fn btree_node_upper_v1<K, V>() -> Result<u64, RetentionContractError> {
    let alignment = 8.max(align_of::<K>()).max(align_of::<V>());
    let fields = size_of::<K>()
        .checked_add(size_of::<V>())
        .and_then(|s| s.checked_mul(11))
        .and_then(|s| s.checked_add(12))
        .and_then(|s| s.checked_add(4usize.checked_mul(alignment - 1)?))
        .ok_or(RetentionContractError::Overflow)?;
    let leaf = align_up_v1(fields, alignment)?;
    let internal = align_up_v1(
        leaf.checked_add(96)
            .ok_or(RetentionContractError::Overflow)?,
        alignment,
    )?;
    u64::try_from(internal).map_err(|_| RetentionContractError::Overflow)
}

/// Certificate for a fresh, private table in an admitted std/hashbrown model only. Never derive
/// this from an arbitrary map's public capacity after removals/tombstones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreshHashStorageV1 {
    buckets: usize,
}

impl FreshHashStorageV1 {
    pub fn for_request<T>(capacity: usize) -> Result<Self, RetentionContractError> {
        let buckets = if capacity == 0 {
            0
        } else if capacity < 15 {
            let minimum = match size_of::<T>() {
                0..=1 => 14,
                2..=3 => 7,
                _ => 3,
            };
            match capacity.max(minimum) {
                0..=3 => 4,
                4..=7 => 8,
                _ => 16,
            }
        } else {
            capacity
                .checked_mul(8)
                .map(|n| n / 7)
                .and_then(|n| n.checked_next_power_of_two())
                .ok_or(RetentionContractError::Overflow)?
        };
        Ok(Self { buckets })
    }
    pub const fn buckets(self) -> usize {
        self.buckets
    }
    pub fn allocation<T>(self) -> Result<u64, RetentionContractError> {
        if self.buckets == 0 {
            return Ok(0);
        }
        let data = self
            .buckets
            .checked_mul(size_of::<T>())
            .ok_or(RetentionContractError::Overflow)?;
        let bytes = align_up_v1(data, align_of::<T>().max(16))?
            .checked_add(self.buckets)
            .and_then(|n| n.checked_add(16))
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(RetentionContractError::Overflow)?;
        u64::try_from(bytes).map_err(|_| RetentionContractError::Overflow)
    }
}

macro_rules! fixed {
    ($($ty:ty),* $(,)?) => {$(
        impl RetainedOwnerV1 for $ty {
            fn retained_bound_v1(&self, _: &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> { Ok(()) }
        }
        unsafe impl ImmutableRetainedOwnerV1 for $ty {}
        unsafe impl FixedRetainedOwnerV1 for $ty {}
    )*};
}
fixed!(
    (),
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    char,
    AtomicBool,
    AtomicU32,
    AtomicU64,
    AtomicUsize,
    &'static str
);

impl RetainedOwnerV1 for str {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        Ok(())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for str {}

impl RetainedOwnerV1 for String {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.array::<u8>(self.capacity())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for String {}

impl RetainedOwnerV1 for OsStr {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        Ok(())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for OsStr {}
impl RetainedOwnerV1 for OsString {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.array::<u8>(self.capacity())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for OsString {}
impl RetainedOwnerV1 for Path {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        Ok(())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for Path {}
impl RetainedOwnerV1 for PathBuf {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.array::<u8>(self.capacity())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for PathBuf {}

impl<T: RetainedOwnerV1> RetainedOwnerV1 for Vec<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.array::<T>(self.capacity())?;
        m.items(self.len())?;
        for v in self {
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for Vec<T> {}
impl<T: RetainedOwnerV1> RetainedOwnerV1 for VecDeque<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.array::<T>(self.capacity())?;
        m.items(self.len())?;
        for v in self {
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for VecDeque<T> {}
impl<T: RetainedOwnerV1> RetainedOwnerV1 for [T] {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.items(self.len())?;
        for v in self {
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for [T] {}
impl<T: RetainedOwnerV1, const N: usize> RetainedOwnerV1 for [T; N] {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.as_slice().retained_bound_v1(m)
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1, const N: usize> ImmutableRetainedOwnerV1 for [T; N] {}
unsafe impl<T: FixedRetainedOwnerV1, const N: usize> FixedRetainedOwnerV1 for [T; N] {}
impl<T: RetainedOwnerV1> RetainedOwnerV1 for Option<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        if let Some(v) = self {
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for Option<T> {}
unsafe impl<T: FixedRetainedOwnerV1> FixedRetainedOwnerV1 for Option<T> {}
impl<T: RetainedOwnerV1 + ?Sized> RetainedOwnerV1 for Box<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.owned(self.as_ref())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1 + ?Sized> ImmutableRetainedOwnerV1 for Box<T> {}
impl<T: ImmutableRetainedOwnerV1 + ?Sized> RetainedOwnerV1 for Arc<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.arc_payload(self.as_ref())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1 + ?Sized> ImmutableRetainedOwnerV1 for Arc<T> {}
impl<T: FixedRetainedOwnerV1> RetainedOwnerV1 for Mutex<T> {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        Ok(())
    }
}
unsafe impl<T: FixedRetainedOwnerV1> ImmutableRetainedOwnerV1 for Mutex<T> {}

impl<K: RetainedOwnerV1, V: RetainedOwnerV1> RetainedOwnerV1 for BTreeMap<K, V> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.btree::<K, V>(self.len())?;
        m.items(self.len())?;
        for (k, v) in self {
            k.retained_bound_v1(m)?;
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<K: ImmutableRetainedOwnerV1, V: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1
    for BTreeMap<K, V>
{
}
impl<T: RetainedOwnerV1> RetainedOwnerV1 for BTreeSet<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.btree::<T, ()>(self.len())?;
        m.items(self.len())?;
        for v in self {
            v.retained_bound_v1(m)?;
        }
        Ok(())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for BTreeSet<T> {}

impl<A: RetainedOwnerV1, B: RetainedOwnerV1> RetainedOwnerV1 for (A, B) {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.0.retained_bound_v1(m)?;
        self.1.retained_bound_v1(m)
    }
}
unsafe impl<A: ImmutableRetainedOwnerV1, B: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1
    for (A, B)
{
}
unsafe impl<A: FixedRetainedOwnerV1, B: FixedRetainedOwnerV1> FixedRetainedOwnerV1 for (A, B) {}

impl RetainedOwnerV1 for serde_json::Value {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.nest(|m| match self {
            Self::String(v) => v.retained_bound_v1(m),
            Self::Array(v) => v.retained_bound_v1(m),
            Self::Object(v) => {
                m.btree::<String, Self>(v.len())?;
                m.items(v.len())?;
                for (k, v) in v {
                    k.retained_bound_v1(m)?;
                    v.retained_bound_v1(m)?;
                }
                Ok(())
            }
            Self::Null | Self::Bool(_) | Self::Number(_) => Ok(()),
        })
    }
}
unsafe impl ImmutableRetainedOwnerV1 for serde_json::Value {}

/// Implements a concrete owner whose only transitive allocations are the named fields.
/// Omitted fields must be allocation-free for every valid value.
#[macro_export]
macro_rules! impl_retained_owner_v1 {
    ($ty:ty { $($field:tt),* $(,)? }) => {
        impl $crate::retained_memory::RetainedOwnerV1 for $ty {
            fn retained_bound_v1(&self, meter: &mut $crate::retained_memory::RetainedBoundBuilderV1)
                -> Result<(), $crate::retained_memory::RetentionContractError> {
                $( $crate::retained_memory::RetainedOwnerV1::retained_bound_v1(&self.$field, meter)?; )*
                let _ = meter;
                Ok(())
            }
        }
    };
}

impl<T> RetainedOwnerV1 for crate::resource::ResourceHandle<T> {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        Ok(())
    }
}
unsafe impl<T> ImmutableRetainedOwnerV1 for crate::resource::ResourceHandle<T> {}
unsafe impl<T> FixedRetainedOwnerV1 for crate::resource::ResourceHandle<T> {}
impl RetainedOwnerV1 for crate::resource::AssetReference {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.locator.retained_bound_v1(m)
    }
}
unsafe impl ImmutableRetainedOwnerV1 for crate::resource::AssetReference {}

/// Private mutation boundary for a shared owner with an enforced lifetime maximum.
/// No mutable guard escapes; replacement is validated before the old owner is moved.
pub struct BoundedRetainedStateV1<T> {
    value: Mutex<T>,
    maximum: RetainedBytesV1,
    model: RetainedAccountingModelV1,
}
impl<T: ImmutableRetainedOwnerV1> BoundedRetainedStateV1<T> {
    pub fn try_new(
        value: T,
        maximum: RetainedBytesV1,
        model: RetainedAccountingModelV1,
    ) -> Result<Self, RetentionContractError> {
        let mut meter = RetainedBoundBuilderV1::with_limit(model, maximum);
        meter.owned(&value)?;
        Ok(Self {
            value: Mutex::new(value),
            maximum,
            model,
        })
    }
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        let value = self.value.lock().unwrap_or_else(|e| e.into_inner());
        read(&value)
    }
    pub fn try_replace(&self, value: T) -> Result<T, RetentionContractError> {
        let mut meter = RetainedBoundBuilderV1::with_limit(self.model, self.maximum);
        meter.owned(&value)?;
        let old = {
            let mut guard = self.value.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::replace(&mut *guard, value)
        };
        Ok(old)
    }
    pub const fn maximum(&self) -> RetainedBytesV1 {
        self.maximum
    }
}
impl<T: ImmutableRetainedOwnerV1> RetainedOwnerV1 for BoundedRetainedStateV1<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        m.add(self.maximum.get())
    }
}
unsafe impl<T: ImmutableRetainedOwnerV1> ImmutableRetainedOwnerV1 for BoundedRetainedStateV1<T> {}

macro_rules! fixed_external {
    ($($t:ty),* $(,)?) => {$(
        impl RetainedOwnerV1 for $t {
            fn retained_bound_v1(&self, _: &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> { Ok(()) }
        }
        unsafe impl ImmutableRetainedOwnerV1 for $t {}
        unsafe impl FixedRetainedOwnerV1 for $t {}
    )*};
}
fixed_external!(
    glam::Vec2,
    glam::Vec3,
    glam::Vec4,
    glam::Quat,
    glam::Mat2,
    glam::Mat3,
    glam::Mat4,
    glam::DVec2,
    glam::DVec3,
    glam::DVec4,
    glam::DQuat,
    glam::DMat3,
    glam::DMat4,
    zr_math::Transform,
    glam::IVec2,
    glam::IVec3,
    glam::IVec4,
    glam::UVec2,
    glam::UVec3,
    glam::UVec4,
    uuid::Uuid
);

#[cfg(test)]
#[path = "retained_memory/tests/cases.rs"]
mod tests;

impl_retained_owner_v1!(crate::resource::ResourceId {});
impl_retained_owner_v1!(crate::resource::AssetUuid {});
impl_retained_owner_v1!(crate::resource::UntypedResourceHandle {});
unsafe impl ImmutableRetainedOwnerV1 for crate::resource::ResourceId {}
unsafe impl ImmutableRetainedOwnerV1 for crate::resource::AssetUuid {}
unsafe impl ImmutableRetainedOwnerV1 for crate::resource::UntypedResourceHandle {}

impl RetainedOwnerV1 for crate::ZrRuntimeOperationSubmitRequestV1 {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.operation_id.retained_bound_v1(m)?;
        self.payload.retained_bound_v1(m)
    }
}

// Weak keeps the Arc allocation itself alive after T is dropped, but does not own T's children.
impl<T> RetainedOwnerV1 for std::sync::Weak<T> {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        if self.as_ptr().is_null() {
            return Ok(());
        }
        let a = std::mem::align_of::<T>();
        let bytes = align_up_v1(16, a)?
            .checked_add(std::mem::size_of::<T>())
            .ok_or(RetentionContractError::Overflow)?;
        m.layout(align_up_v1(bytes, a.max(8))?, a.max(8))
    }
}
unsafe impl<T> ImmutableRetainedOwnerV1 for std::sync::Weak<T> {}
