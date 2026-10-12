use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::super::command::{ChromeCommand, ChromeCommandKind};
use crate::ui::retained_host::host_contract::paint_template_nodes::copy_editor_sprite_atlas_rgba;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageResource {
    pub(in crate::ui::retained_host::host_contract) generation: u64,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) rgba: Arc<[u8]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageResources {
    by_resource_key: HashMap<String, BTreeMap<u64, ChromeImageResource>>,
}

impl ChromeImageResources {
    pub(in crate::ui::retained_host::host_contract) fn insert(
        &mut self,
        resource_key: String,
        resource: ChromeImageResource,
    ) {
        self.by_resource_key
            .entry(resource_key)
            .or_default()
            .insert(resource.generation, resource);
    }

    pub(in crate::ui::retained_host::host_contract) fn get(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<&ChromeImageResource> {
        self.by_resource_key
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
    }

    pub(in crate::ui::retained_host::host_contract) fn is_empty(&self) -> bool {
        self.by_resource_key.is_empty()
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host::host_contract) fn len(&self) -> usize {
        self.by_resource_key.values().map(BTreeMap::len).sum()
    }

    pub(in crate::ui::retained_host::host_contract) fn retain(
        &mut self,
        mut keep: impl FnMut(&str, u64, &ChromeImageResource) -> bool,
    ) {
        self.by_resource_key.retain(|resource_key, generations| {
            generations.retain(|generation, resource| keep(resource_key, *generation, resource));
            !generations.is_empty()
        });
    }

    pub(in crate::ui::retained_host::host_contract) fn iter(
        &self,
    ) -> impl Iterator<Item = (&str, u64, &ChromeImageResource)> {
        self.by_resource_key
            .iter()
            .flat_map(|(resource_key, generations)| {
                generations.iter().map(move |(generation, resource)| {
                    (resource_key.as_str(), *generation, resource)
                })
            })
    }

    pub(in crate::ui::retained_host::host_contract) fn into_entries(
        self,
    ) -> impl Iterator<Item = (String, ChromeImageResource)> {
        self.by_resource_key
            .into_iter()
            .flat_map(|(resource_key, generations)| {
                generations
                    .into_values()
                    .map(move |resource| (resource_key.clone(), resource))
            })
    }

    pub(in crate::ui::retained_host::host_contract) fn extend(&mut self, resources: Self) {
        for (resource_key, mut generations) in resources.by_resource_key {
            self.by_resource_key
                .entry(resource_key)
                .or_default()
                .append(&mut generations);
        }
    }
}

pub(super) fn compact_image_resources(commands: &mut [ChromeCommand]) -> ChromeImageResources {
    compact_image_resources_with_residency(commands, |_, _| false)
}

pub(super) fn compact_image_resources_with_residency(
    commands: &mut [ChromeCommand],
    mut is_resident: impl FnMut(&str, u64) -> bool,
) -> ChromeImageResources {
    let mut resources = ChromeImageResources::default();
    let mut resident_results = HashMap::<String, BTreeMap<u64, bool>>::new();
    for command in commands {
        let ChromeCommandKind::Image { payload } = &mut command.kind else {
            continue;
        };
        let resident = resident_results
            .get(payload.resource_key.as_str())
            .and_then(|generations| generations.get(&payload.resource_generation))
            .copied()
            .unwrap_or_else(|| {
                let resident =
                    is_resident(payload.resource_key.as_str(), payload.resource_generation);
                resident_results
                    .entry(payload.resource_key.clone())
                    .or_default()
                    .insert(payload.resource_generation, resident);
                resident
            });
        if resident {
            payload.rgba = None;
            continue;
        }
        let needs_resource = resources
            .get(payload.resource_key.as_str(), payload.resource_generation)
            .is_none();
        let rgba = payload.rgba.take().or_else(|| {
            (needs_resource && payload.atlas_uv.is_some()).then(|| {
                copy_editor_sprite_atlas_rgba(
                    payload.resource_key.as_str(),
                    payload.resource_generation,
                )
                .map(Arc::from)
            })?
        });
        let Some(rgba) = rgba else {
            continue;
        };
        if !needs_resource {
            continue;
        }
        resources.insert(
            payload.resource_key.clone(),
            ChromeImageResource {
                generation: payload.resource_generation,
                width: payload.width,
                height: payload.height,
                upload_bytes: payload.upload_bytes,
                rgba,
            },
        );
    }
    resources
}

#[cfg(test)]
#[path = "tests/image_resources.rs"]
mod tests;
