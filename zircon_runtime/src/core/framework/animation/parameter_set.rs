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
pub struct AnimationParameterRevision(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
mod tests {
    use super::*;

    #[test]
    fn clone_shares_values_until_content_changes() {
        let mut source =
            AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
        let original = source.clone();
        let original_revision = source.revision();

        source.insert("speed".into(), AnimationParameterValue::Scalar(0.75));

        assert_ne!(source.revision(), original_revision);
        assert_eq!(
            original.get("speed"),
            Some(&AnimationParameterValue::Scalar(0.25))
        );
        assert_eq!(
            source.get("speed"),
            Some(&AnimationParameterValue::Scalar(0.75))
        );
    }

    #[test]
    fn iterator_collection_constructs_one_revisioned_owner() {
        let parameters: AnimationParameterSet = [
            ("speed".into(), AnimationParameterValue::Scalar(0.25)),
            ("grounded".into(), AnimationParameterValue::Bool(true)),
        ]
        .into_iter()
        .collect();
        let cloned = parameters.clone();

        assert_eq!(parameters.len(), 2);
        assert_eq!(cloned.revision(), parameters.revision());
        assert_eq!(
            cloned.content_fingerprint(),
            parameters.content_fingerprint()
        );
        assert!(Arc::ptr_eq(&cloned.values, &parameters.values));
    }

    #[test]
    fn equal_insert_and_missing_remove_preserve_revision() {
        let mut parameters =
            AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
        let revision = parameters.revision();

        assert_eq!(
            parameters.insert("speed".into(), AnimationParameterValue::Scalar(0.25)),
            Some(AnimationParameterValue::Scalar(0.25))
        );
        assert_eq!(parameters.remove("missing"), None);
        assert_eq!(parameters.revision(), revision);
    }

    #[test]
    fn serialization_reconstructs_runtime_revision_without_changing_values() {
        let parameters =
            AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
        let encoded = serde_json::to_vec(&parameters).unwrap();

        let decoded: AnimationParameterSet = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded, parameters);
        assert_ne!(decoded.revision(), parameters.revision());
        assert_eq!(
            decoded.content_fingerprint(),
            parameters.content_fingerprint()
        );
    }

    #[test]
    fn content_fingerprint_tracks_mutation_and_normalizes_signed_zero() {
        let positive_zero = AnimationParameterSet::from([
            ("scalar".into(), AnimationParameterValue::Scalar(0.0)),
            (
                "vector".into(),
                AnimationParameterValue::Vec4([0.0, 1.0, 2.0, 3.0]),
            ),
        ]);
        let mut negative_zero = AnimationParameterSet::from([
            ("scalar".into(), AnimationParameterValue::Scalar(-0.0)),
            (
                "vector".into(),
                AnimationParameterValue::Vec4([-0.0, 1.0, 2.0, 3.0]),
            ),
        ]);

        assert_eq!(positive_zero, negative_zero);
        assert_eq!(
            positive_zero.content_fingerprint(),
            negative_zero.content_fingerprint()
        );

        negative_zero.insert("scalar".into(), AnimationParameterValue::Scalar(1.0));
        assert_ne!(positive_zero, negative_zero);
        assert_ne!(
            positive_zero.content_fingerprint(),
            negative_zero.content_fingerprint()
        );
    }

    #[test]
    fn content_fingerprint_collision_still_requires_value_equality() {
        let left =
            AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
        let mut right =
            AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.75))]);
        right.content_fingerprint = left.content_fingerprint;

        assert_ne!(left, right);
    }
}
