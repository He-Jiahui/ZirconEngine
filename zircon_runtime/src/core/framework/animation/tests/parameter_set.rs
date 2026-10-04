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
