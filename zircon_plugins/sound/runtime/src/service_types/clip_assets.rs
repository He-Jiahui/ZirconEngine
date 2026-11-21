//! clip 装载经项目资产管理器解析定位符并缓存 Kira 数据；脱离 Core 的测试实例只可使用测试插入入口。

use zircon_runtime::asset::pipeline::manager::ProjectGenerationCommitOutcome;
use zircon_runtime::asset::project::ProjectActivationIdentity;
#[cfg(test)]
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::asset::{project_asset_manager_handle, AssetUri, ProjectAssetManager};
use zircon_runtime::core::framework::sound::{SoundClipId, SoundClipInfo, SoundError};
use zircon_runtime::core::manager::resolve_manager_service;
use zircon_runtime::core::resource::ResourceScheme;
use zircon_runtime::core::ServiceCallGuard;

use crate::engine::LoadedClip;

use super::DefaultSoundManager;

const MAX_PROJECT_CLIP_LOAD_ATTEMPTS: usize = 2;

impl DefaultSoundManager {
    fn project_asset_manager(&self) -> Result<ServiceCallGuard<ProjectAssetManager>, SoundError> {
        let core = self
            .core
            .as_ref()
            .and_then(|core| core.upgrade())
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: "sound manager is not attached to a CoreRuntime".to_string(),
            })?;
        project_asset_manager_handle(&core)
            .and_then(|handle| resolve_manager_service(&core, handle))
            .map_err(|error| SoundError::BackendUnavailable {
                detail: error.to_string(),
            })
    }

    #[cfg(test)]
    pub(crate) fn insert_clip_for_test(&self, asset: SoundAsset) -> SoundClipId {
        let loaded = LoadedClip::new(asset).expect("test sound asset must map to Kira M1 data");
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state.clips.insert(clip_id, loaded);
        clip_id
    }

    pub(super) fn load_clip_impl(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        let uri = AssetUri::parse(locator).map_err(|_| SoundError::InvalidLocator {
            locator: locator.to_string(),
        })?;
        // Drop the sound-state guard before consulting the project authority.
        let cached = crate::poison_recovery::lock_recover(&self.state)
            .clip_ids_by_locator
            .get(locator)
            .cloned();
        if uri.scheme() != ResourceScheme::Res {
            if let Some((clip, _)) = cached {
                return Ok(clip);
            }
            let assets = self.project_asset_manager()?;
            let loaded = Self::prepare_clip(&assets, &uri, locator)?;
            return Ok(self.publish_clip(locator, loaded, None).0);
        }

        let assets = self.project_asset_manager()?;
        if let Some((clip, Some(activation))) = cached {
            // A valid hit needs neither a ProjectManager snapshot nor a PCM read/copy.
            if assets.is_project_activation_current(&activation) {
                return Ok(clip);
            }
        }
        for _ in 0..MAX_PROJECT_CLIP_LOAD_ATTEMPTS {
            let (_, generation) = assets
                .current_project_generation_snapshot()
                .ok_or(SoundError::NoProjectOpen)?
                .into_parts();
            // Resolve and read without either the project commit fence or the sound lock.
            // Even errors from a superseded lookup should be retried under the new activation.
            let prepared = Self::prepare_clip(&assets, &uri, locator);
            let outcome = assets.commit_if_project_generation(&generation, || {
                prepared.map(|loaded| {
                    self.publish_clip(locator, loaded, Some(generation.activation_identity()))
                })
            });
            if let ProjectGenerationCommitOutcome::Committed(result) = outcome {
                // A concurrent loser discards its PCM outside both the generation fence and lock.
                return result.map(|(clip, _discarded)| clip);
            }
        }
        Err(SoundError::Decode(format!(
            "sound asset {locator} load was superseded by project generation changes"
        )))
    }

    fn prepare_clip(
        assets: &ProjectAssetManager,
        uri: &AssetUri,
        locator: &str,
    ) -> Result<LoadedClip, SoundError> {
        let asset_id = assets
            .resolve_asset_id(uri)
            .ok_or_else(|| SoundError::InvalidLocator {
                locator: locator.to_string(),
            })?;
        let asset = assets
            .load_sound_asset(asset_id)
            .map_err(|error| SoundError::Decode(error.to_string()))?;
        LoadedClip::new(asset)
    }

    fn publish_clip(
        &self,
        locator: &str,
        loaded: LoadedClip,
        activation: Option<&ProjectActivationIdentity>,
    ) -> (SoundClipId, Option<LoadedClip>) {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        if let Some((clip, cached_activation)) = state.clip_ids_by_locator.get(locator) {
            if cached_activation.as_ref() == activation {
                return (*clip, Some(loaded));
            }
        }

        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state
            .clip_ids_by_locator
            .insert(locator.to_string(), (clip_id, activation.cloned()));
        // Retire discovery, not the handle or frames already supplied to the caller.
        state.clips.insert(clip_id, loaded);
        (clip_id, None)
    }

    pub(super) fn clip_info_impl(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        let state = crate::poison_recovery::lock_recover(&self.state);
        let clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        Ok(SoundClipInfo {
            locator: clip.asset.uri.to_string(),
            sample_rate_hz: clip.asset.sample_rate_hz,
            channel_count: clip.asset.channel_count,
            channel_layout: clip.asset.channel_layout.clone(),
            frame_count: clip.asset.frame_count(),
            duration_seconds: clip.asset.duration_seconds(),
        })
    }
}

#[cfg(test)]
#[path = "tests/clip_assets.rs"]
mod tests;
