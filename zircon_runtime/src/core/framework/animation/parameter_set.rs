use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{AnimationParameterMap, AnimationParameterValue};

static NEXT_ANIMATION_PARAMETER_REVISION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 参数内容发生变化时分配的新运行期版本，用于使状态机采样缓存失效；序列化不会保留此标识。
pub struct AnimationParameterRevision(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 参数映射内容的快速索引摘要；缓存命中仍应核对完整映射，以免摘要碰撞复用错误结果。
pub struct AnimationParameterContentFingerprint(u64);

impl AnimationParameterRevision {
    fn next() -> Self {
        let revision = NEXT_ANIMATION_PARAMETER_REVISION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |revision| {
                revision.checked_add(1)
            })
            .expect("animation parameter revision space exhausted");
        Self(revision)
    }
}

#[derive(Clone)]
/// 可共享的参数映射；克隆共享底层数据，内容变化时才触发写时复制并更新版本和摘要。
/// 序列化只保存映射内容，因此反序列化后会建立新的运行期版本。
pub struct AnimationParameterSet {
    values: Arc<AnimationParameterMap>,
    revision: AnimationParameterRevision,
    content_fingerprint: AnimationParameterContentFingerprint,
}

impl AnimationParameterSet {
    pub fn new() -> Self {
        Self::from(AnimationParameterMap::new())
    }

    pub fn revision(&self) -> AnimationParameterRevision {
        self.revision
    }

    pub fn content_fingerprint(&self) -> AnimationParameterContentFingerprint {
        self.content_fingerprint
    }

    pub fn as_map(&self) -> &AnimationParameterMap {
        self.values.as_ref()
    }

    pub fn into_map(self) -> AnimationParameterMap {
        Arc::try_unwrap(self.values).unwrap_or_else(|values| values.as_ref().clone())
    }

    pub fn insert(
        &mut self,
        name: String,
        value: AnimationParameterValue,
    ) -> Option<AnimationParameterValue> {
        if self.values.get(&name) == Some(&value) {
            return Some(value);
        }
        let previous = Arc::make_mut(&mut self.values).insert(name, value);
        self.revision = AnimationParameterRevision::next();
        self.refresh_content_fingerprint();
        previous
    }

    pub fn update_existing<Q>(&mut self, name: &Q, value: AnimationParameterValue) -> bool
    where
        String: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let Some(current) = self.values.get(name) else {
            return false;
        };
        if current != &value {
            *Arc::make_mut(&mut self.values)
                .get_mut(name)
                .expect("existing parameter") = value;
            self.revision = AnimationParameterRevision::next();
            self.refresh_content_fingerprint();
        }
        true
    }

    pub fn remove<Q>(&mut self, name: &Q) -> Option<AnimationParameterValue>
    where
        String: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if !self.values.contains_key(name) {
            return None;
        }
        let removed = Arc::make_mut(&mut self.values).remove(name);
        self.revision = AnimationParameterRevision::next();
        self.refresh_content_fingerprint();
        removed
    }

    pub fn clear(&mut self) {
        if self.values.is_empty() {
            return;
        }
        Arc::make_mut(&mut self.values).clear();
        self.revision = AnimationParameterRevision::next();
        self.refresh_content_fingerprint();
    }

    fn refresh_content_fingerprint(&mut self) {
        self.content_fingerprint = parameter_content_fingerprint(self.values.as_ref());
    }
}

impl Default for AnimationParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AnimationParameterSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.values.fmt(formatter)
    }
}

impl PartialEq for AnimationParameterSet {
    fn eq(&self, other: &Self) -> bool {
        if self.content_fingerprint != other.content_fingerprint {
            return false;
        }
        Arc::ptr_eq(&self.values, &other.values) || self.values == other.values
    }
}

impl Deref for AnimationParameterSet {
    type Target = AnimationParameterMap;

    fn deref(&self) -> &Self::Target {
        self.as_map()
    }
}

impl From<AnimationParameterMap> for AnimationParameterSet {
    fn from(values: AnimationParameterMap) -> Self {
        let content_fingerprint = parameter_content_fingerprint(&values);
        Self {
            values: Arc::new(values),
            revision: AnimationParameterRevision::next(),
            content_fingerprint,
        }
    }
}

impl FromIterator<(String, AnimationParameterValue)> for AnimationParameterSet {
    fn from_iter<T: IntoIterator<Item = (String, AnimationParameterValue)>>(values: T) -> Self {
        Self::from(values.into_iter().collect::<AnimationParameterMap>())
    }
}

fn parameter_content_fingerprint(
    values: &AnimationParameterMap,
) -> AnimationParameterContentFingerprint {
    let mut hasher = DefaultHasher::new();
    values.len().hash(&mut hasher);
    for (name, value) in values {
        name.hash(&mut hasher);
        std::mem::discriminant(value).hash(&mut hasher);
        match value {
            AnimationParameterValue::Bool(value) => value.hash(&mut hasher),
            AnimationParameterValue::Integer(value) => value.hash(&mut hasher),
            AnimationParameterValue::Scalar(value) => hash_parameter_float(*value, &mut hasher),
            AnimationParameterValue::Vec2(values) => hash_parameter_floats(values, &mut hasher),
            AnimationParameterValue::Vec3(values) => hash_parameter_floats(values, &mut hasher),
            AnimationParameterValue::Vec4(values) => hash_parameter_floats(values, &mut hasher),
            AnimationParameterValue::Trigger => {}
        }
    }
    AnimationParameterContentFingerprint(hasher.finish())
}

fn hash_parameter_floats<const N: usize>(values: &[f32; N], hasher: &mut impl Hasher) {
    for value in values {
        hash_parameter_float(*value, hasher);
    }
}

fn hash_parameter_float(value: f32, hasher: &mut impl Hasher) {
    let bits = if value == 0.0 { 0 } else { value.to_bits() };
    bits.hash(hasher);
}

impl<const N: usize> From<[(String, AnimationParameterValue); N]> for AnimationParameterSet {
    fn from(values: [(String, AnimationParameterValue); N]) -> Self {
        Self::from(AnimationParameterMap::from(values))
    }
}

impl Serialize for AnimationParameterSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.values.as_ref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnimationParameterSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AnimationParameterMap::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
#[path = "tests/parameter_set.rs"]
mod tests;
