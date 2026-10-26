use std::collections::BTreeSet;

use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::math::Real;

use super::{
    ParticleSourceEmitter, ParticleSourceError, ParticleSourceScalarRange, ParticleSourceShape,
    ParticleSystemSource,
};

impl ParticleSystemSource {
    /// Source commits reject invalid values rather than hiding runtime normalization/clamping.
    /// Empty systems are valid authoring assets; the preview host must skip instantiation.
    pub fn validate(&self) -> Result<(), ParticleSourceError> {
        if self.format_version != 1 {
            return Err(ParticleSourceError::invalid(
                "format_version",
                "unsupported source version",
            ));
        }
        nonempty("system.id", &self.system.id)?;
        let mut keys = BTreeSet::new();
        let mut emitter_ids = BTreeSet::new();
        for emitter in &self.emitters {
            let key = require_key(&mut keys, emitter.key.as_deref(), "emitters.key")?;
            let path = format!("emitters[{key}]");
            nonempty(&format!("{path}.id"), &emitter.id)?;
            if !emitter_ids.insert(emitter.id.as_str()) {
                return Err(ParticleSourceError::invalid(
                    format!("{path}.id"),
                    "duplicate runtime emitter id",
                ));
            }
            validate_emitter(emitter, &path, &mut keys)?;
        }
        Ok(())
    }
}

fn validate_emitter<'a>(
    emitter: &'a ParticleSourceEmitter,
    path: &str,
    keys: &mut BTreeSet<&'a str>,
) -> Result<(), ParticleSourceError> {
    // Capacity zero does not bypass source validation, unlike the current runtime validator.
    nonnegative(
        &format!("{path}.spawn_rate_per_second"),
        emitter.spawn_rate_per_second,
    )?;
    nonnegative(&format!("{path}.drag"), emitter.drag)?;
    range(
        &format!("{path}.lifetime"),
        emitter.lifetime,
        Some(Real::MIN_POSITIVE),
    )?;
    range(
        &format!("{path}.initial_size"),
        emitter.initial_size,
        Some(0.0),
    )?;
    range(
        &format!("{path}.initial_rotation"),
        emitter.initial_rotation,
        None,
    )?;
    range(
        &format!("{path}.initial_angular_velocity"),
        emitter.initial_angular_velocity,
        None,
    )?;
    for axis in 0..3 {
        range(
            &format!("{path}.initial_velocity[{axis}]"),
            ParticleSourceScalarRange {
                min: emitter.initial_velocity.min[axis],
                max: emitter.initial_velocity.max[axis],
            },
            None,
        )?;
    }
    finite_values(&format!("{path}.gravity"), &emitter.gravity)?;
    finite_values(&format!("{path}.start_color"), &emitter.start_color)?;
    match emitter.shape {
        ParticleSourceShape::Point => {}
        ParticleSourceShape::Sphere { radius } => {
            nonnegative(&format!("{path}.shape.radius"), radius)?
        }
        ParticleSourceShape::Box { half_extents } => {
            for (axis, value) in half_extents.into_iter().enumerate() {
                nonnegative(&format!("{path}.shape.half_extents[{axis}]"), value)?;
            }
        }
        ParticleSourceShape::Cone { radius, height } => {
            nonnegative(&format!("{path}.shape.radius"), radius)?;
            nonnegative(&format!("{path}.shape.height"), height)?;
        }
    }
    finite_values(
        &format!("{path}.physics.external_force"),
        &emitter.physics.external_force,
    )?;
    nonnegative(&format!("{path}.physics.bounce"), emitter.physics.bounce)?;
    normalized(&format!("{path}.physics.damping"), emitter.physics.damping)?;
    for (index, binding) in emitter.animation_bindings.iter().enumerate() {
        nonempty(
            &format!("{path}.animation_bindings[{index}].parameter"),
            &binding.parameter,
        )?;
        nonempty(
            &format!("{path}.animation_bindings[{index}].curve_path"),
            &binding.curve_path,
        )?;
        normalized(
            &format!("{path}.animation_bindings[{index}].normalized_progress"),
            binding.normalized_progress,
        )?;
    }
    for (field, uri) in [
        ("material", &emitter.material),
        ("texture", &emitter.texture),
    ] {
        if let Some(uri) = uri {
            AssetUri::parse(uri).map_err(|error| {
                ParticleSourceError::invalid(format!("{path}.{field}"), error.to_string())
            })?;
        }
    }
    for burst in &emitter.bursts {
        let key = require_key(keys, burst.key.as_deref(), &format!("{path}.bursts.key"))?;
        nonnegative(
            &format!("{path}.bursts[{key}].time_seconds"),
            burst.time_seconds,
        )?;
    }
    let mut previous = None;
    for key in &emitter.size_over_lifetime {
        let id = require_key(
            keys,
            key.key.as_deref(),
            &format!("{path}.size_over_lifetime.key"),
        )?;
        let path = format!("{path}.size_over_lifetime[{id}]");
        ordered_time(&path, key.t, &mut previous)?;
        nonnegative(&format!("{path}.value"), key.value)?;
    }
    let mut previous = None;
    for key in &emitter.color_over_lifetime {
        let id = require_key(
            keys,
            key.key.as_deref(),
            &format!("{path}.color_over_lifetime.key"),
        )?;
        let path = format!("{path}.color_over_lifetime[{id}]");
        ordered_time(&path, key.t, &mut previous)?;
        finite_values(&format!("{path}.value"), &key.value)?;
    }
    Ok(())
}

fn require_key<'a>(
    keys: &mut BTreeSet<&'a str>,
    key: Option<&'a str>,
    path: &str,
) -> Result<&'a str, ParticleSourceError> {
    let key =
        key.ok_or_else(|| ParticleSourceError::invalid(path, "missing persistent authoring key"))?;
    if key.trim().is_empty() || key.trim() != key || !keys.insert(key) {
        return Err(ParticleSourceError::invalid(
            path,
            "keys must be nonempty and unique",
        ));
    }
    Ok(key)
}

fn nonempty(path: &str, value: &str) -> Result<(), ParticleSourceError> {
    if value.trim().is_empty() {
        return Err(ParticleSourceError::invalid(path, "value cannot be empty"));
    }
    Ok(())
}

fn finite(path: &str, value: Real) -> Result<(), ParticleSourceError> {
    if !value.is_finite() {
        return Err(ParticleSourceError::invalid(path, "value must be finite"));
    }
    Ok(())
}

fn finite_values(path: &str, values: &[Real]) -> Result<(), ParticleSourceError> {
    for (index, value) in values.iter().enumerate() {
        finite(&format!("{path}[{index}]"), *value)?;
    }
    Ok(())
}

pub(super) fn nonnegative(path: &str, value: Real) -> Result<(), ParticleSourceError> {
    finite(path, value)?;
    if value < 0.0 {
        return Err(ParticleSourceError::invalid(
            path,
            "value must be nonnegative",
        ));
    }
    Ok(())
}

fn normalized(path: &str, value: Real) -> Result<(), ParticleSourceError> {
    finite(path, value)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(ParticleSourceError::invalid(
            path,
            "value must be within [0, 1]",
        ));
    }
    Ok(())
}

pub(super) fn range(
    path: &str,
    value: ParticleSourceScalarRange,
    minimum: Option<Real>,
) -> Result<(), ParticleSourceError> {
    finite(path, value.min)?;
    finite(path, value.max)?;
    if value.min > value.max || minimum.is_some_and(|minimum| value.min < minimum) {
        return Err(ParticleSourceError::invalid(
            path,
            "invalid or reversed range",
        ));
    }
    Ok(())
}

fn ordered_time(
    path: &str,
    time: Real,
    previous: &mut Option<Real>,
) -> Result<(), ParticleSourceError> {
    normalized(&format!("{path}.t"), time)?;
    if previous.is_some_and(|previous| time <= previous) {
        return Err(ParticleSourceError::invalid(
            path,
            "curve times must be strictly increasing",
        ));
    }
    *previous = Some(time);
    Ok(())
}
