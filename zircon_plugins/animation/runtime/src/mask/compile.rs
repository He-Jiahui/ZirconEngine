use zircon_runtime::core::math::Real;

use crate::{AnimationClipCompileError, SkeletonTargetTable};

use super::{AvatarMaskAsset, AvatarMaskError, AvatarMaskRule};

#[derive(Clone, Debug, PartialEq)]
pub struct MaskWeights {
    weights: Box<[Real]>,
}

impl MaskWeights {
    pub(crate) fn from_validated_weights(weights: &[Real]) -> Self {
        debug_assert!(weights
            .iter()
            .all(|weight| weight.is_finite() && (0.0..=1.0).contains(weight)));
        Self {
            weights: weights.into(),
        }
    }

    pub fn try_from_weights(weights: Vec<Real>) -> Result<Self, AvatarMaskError> {
        for weight in &weights {
            validate_weight(None, *weight)?;
        }
        Ok(Self {
            weights: weights.into_boxed_slice(),
        })
    }

    pub fn compile(
        asset: &AvatarMaskAsset,
        targets: &SkeletonTargetTable,
    ) -> Result<Self, AvatarMaskError> {
        if asset.id.trim().is_empty() {
            return Err(AvatarMaskError::InvalidId);
        }
        validate_weight(None, asset.default_weight)?;
        let mut weights = vec![asset.default_weight; targets.len()];
        for rule in &asset.rules {
            apply_rule(&mut weights, targets, rule)?;
        }
        Ok(Self {
            weights: weights.into_boxed_slice(),
        })
    }

    pub fn len(&self) -> usize {
        self.weights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    pub fn as_slice(&self) -> &[Real] {
        &self.weights
    }

    pub fn weight(&self, bone_index: usize) -> Option<Real> {
        self.weights.get(bone_index).copied()
    }
}

fn apply_rule(
    weights: &mut [Real],
    targets: &SkeletonTargetTable,
    rule: &AvatarMaskRule,
) -> Result<(), AvatarMaskError> {
    validate_weight(Some(&rule.target), rule.weight)?;
    for weight in &rule.boundary_weights {
        validate_weight(Some(&rule.target), *weight)?;
    }
    let root_path = resolve_rule_root(targets, &rule.target)?;
    for bone_index in 0..targets.len() {
        let Some(path) = targets.bone_path_for_index(bone_index) else {
            continue;
        };
        let Some(depth) = descendant_depth(root_path, path) else {
            continue;
        };
        if depth > 0 && !rule.inherit {
            continue;
        }
        weights[bone_index] = rule
            .boundary_weights
            .get(depth.min(rule.boundary_weights.len().saturating_sub(1)))
            .copied()
            .unwrap_or(rule.weight);
    }
    Ok(())
}

fn resolve_rule_root<'a>(
    targets: &'a SkeletonTargetTable,
    target: &str,
) -> Result<&'a str, AvatarMaskError> {
    let target = target.trim();
    if target.is_empty() {
        return Err(AvatarMaskError::InvalidTarget {
            target: target.to_string(),
        });
    }
    if target.contains('/') {
        return targets
            .bone_paths()
            .iter()
            .find(|path| path.as_str() == target)
            .map(String::as_str)
            .ok_or_else(|| AvatarMaskError::UnresolvedTarget {
                target: target.to_string(),
            });
    }
    let slot = targets
        .resolve_unique_bone_name(0, target)
        .map_err(|error| match error {
            AnimationClipCompileError::AmbiguousTrack { .. } => AvatarMaskError::AmbiguousTarget {
                target: target.to_string(),
            },
            _ => AvatarMaskError::UnresolvedTarget {
                target: target.to_string(),
            },
        })?;
    let index =
        targets
            .bone_index_for_slot(slot)
            .ok_or_else(|| AvatarMaskError::UnresolvedTarget {
                target: target.to_string(),
            })?;
    targets
        .bone_path_for_index(index)
        .ok_or_else(|| AvatarMaskError::UnresolvedTarget {
            target: target.to_string(),
        })
}

fn descendant_depth(root: &str, path: &str) -> Option<usize> {
    if path == root {
        return Some(0);
    }
    let suffix = path.strip_prefix(root)?.strip_prefix('/')?;
    Some(suffix.split('/').count())
}

fn validate_weight(target: Option<&str>, weight: Real) -> Result<(), AvatarMaskError> {
    if weight.is_finite() && (0.0..=1.0).contains(&weight) {
        return Ok(());
    }
    Err(AvatarMaskError::InvalidWeight {
        target: target.map(ToOwned::to_owned),
        weight,
    })
}
