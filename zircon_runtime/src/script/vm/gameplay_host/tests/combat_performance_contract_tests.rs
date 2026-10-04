use crate::core::framework::animation::{
    AnimationParameterMap, AnimationParameterSet, AnimationParameterValue,
};

use super::set_animation_bool_parameter;

#[test]
fn animation_bool_parameter_only_copies_a_missing_key() {
    let mut parameters = AnimationParameterSet::from(AnimationParameterMap::from([(
        "moving".to_owned(),
        AnimationParameterValue::Bool(false),
    )]));
    let original = parameters.clone();
    let revision = parameters.revision();

    assert!(!set_animation_bool_parameter(
        &mut parameters,
        "moving",
        true
    ));
    assert_ne!(parameters.revision(), revision);
    assert_eq!(
        original.get("moving"),
        Some(&AnimationParameterValue::Bool(false))
    );
    let unchanged = parameters.revision();
    assert!(!set_animation_bool_parameter(
        &mut parameters,
        "moving",
        true
    ));
    assert_eq!(parameters.revision(), unchanged);
    assert!(set_animation_bool_parameter(
        &mut parameters,
        "grounded",
        true
    ));
    assert_eq!(
        parameters.get("moving"),
        Some(&AnimationParameterValue::Bool(true))
    );
    assert_eq!(
        parameters.get("grounded"),
        Some(&AnimationParameterValue::Bool(true))
    );
}
