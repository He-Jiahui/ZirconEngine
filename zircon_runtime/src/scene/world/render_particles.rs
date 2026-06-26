use crate::core::framework::render::{
    ParticleExtract, RenderLayerSet, RenderParticleBoundsSnapshot, RenderParticleGpuFrameExtract,
    RenderParticleSpriteSnapshot,
};
use crate::core::math::{Vec2, Vec3, Vec4};
use crate::scene::components::default_render_layer_mask;
use crate::scene::EntityId;

use super::World;

// Gameplay scripts author transient particles as dynamic JSON so runtime-only fallback and
// project scripts share the same render extraction path without spawning mesh entities.
const PARTICLE_COMPONENT_IDS: [&str; 2] = ["render.particle_sprites", "gameplay.particle_sprites"];
const WORLD_HUD_BAR_COMPONENT_IDS: [&str; 2] = ["render.world_hud_bars", "gameplay.world_hud_bars"];
const WORLD_HUD_DEFAULT_WIDTH: f32 = 1.1;
const WORLD_HUD_DEFAULT_HEIGHT: f32 = 0.1;
const WORLD_HUD_BACK_COLOR: Vec4 = Vec4::new(0.04, 0.035, 0.04, 0.72);
const WORLD_HUD_FILL_COLOR: Vec4 = Vec4::new(0.9, 0.08, 0.14, 0.9);
const WORLD_HUD_BACKGROUND_SORT_ORDER: i32 = 10;
const WORLD_HUD_FILL_SORT_ORDER: i32 = 11;
const WORLD_HUD_BAR_KEY_STRIDE: u64 = 2;
const WORLD_HUD_BAR_BACKGROUND_KEY_OFFSET: u64 = 1;
const WORLD_HUD_BAR_FILL_KEY_OFFSET: u64 = 2;

impl World {
    pub(super) fn collect_render_particles(
        &self,
        camera_layers: &RenderLayerSet,
        camera_position: Vec3,
    ) -> ParticleExtract {
        let mut emitters = Vec::new();
        let mut sprites = Vec::new();
        let mut bounds = Vec::new();
        let mut gpu_frame_builder = ParticleGpuFrameBuilder::default();

        for entity in self.entities.iter().copied() {
            if self.active_in_hierarchy(entity) != Some(true)
                || !self.entity_intersects_camera_layers(entity, camera_layers)
            {
                continue;
            }
            let render_layer_mask = self
                .render_layer_mask(entity)
                .unwrap_or(default_render_layer_mask());
            let mut entity_sprites = Vec::new();
            let mut entity_gpu_bounds = Vec::new();
            let mut has_gpu_frame = false;
            for component_id in PARTICLE_COMPONENT_IDS {
                let Some(value) = self.dynamic_component(entity, component_id) else {
                    continue;
                };
                collect_particle_sprites_from_value(
                    entity,
                    render_layer_mask,
                    value,
                    &mut entity_sprites,
                );
                if let Some(contribution) = particle_gpu_frame_contribution(value) {
                    has_gpu_frame = true;
                    if let Some(bound) = contribution.bounds {
                        entity_gpu_bounds.push(bound);
                    }
                    gpu_frame_builder.push(contribution.frame);
                }
            }
            for component_id in WORLD_HUD_BAR_COMPONENT_IDS {
                let Some(value) = self.dynamic_component(entity, component_id) else {
                    continue;
                };
                collect_world_hud_bar_sprites_from_value(
                    entity,
                    render_layer_mask,
                    value,
                    &mut entity_sprites,
                );
            }
            if entity_sprites.is_empty() && !has_gpu_frame {
                continue;
            }

            let center = self
                .world_transform(entity)
                .map(|transform| transform.translation)
                .or_else(|| entity_gpu_bounds.first().map(|bound| bound.center))
                .unwrap_or(camera_position);
            let sprite_radius = entity_sprites
                .iter()
                .map(|sprite| (sprite.position - center).length() + sprite.size)
                .filter(|value| value.is_finite())
                .fold(0.0_f32, f32::max);
            let gpu_radius = entity_gpu_bounds
                .iter()
                .map(|bound| (bound.center - center).length() + bound.radius)
                .filter(|value| value.is_finite())
                .fold(0.0_f32, f32::max);
            emitters.push(entity);
            bounds.push(RenderParticleBoundsSnapshot {
                entity,
                center,
                radius: sprite_radius.max(gpu_radius).max(0.01),
            });
            sprites.extend(entity_sprites);
        }

        emitters.sort_unstable();
        sprites.sort_by(|left, right| {
            distance_sort_key(right.position, camera_position)
                .total_cmp(&distance_sort_key(left.position, camera_position))
                .then_with(|| left.entity.cmp(&right.entity))
                .then_with(|| left.sort_order.cmp(&right.sort_order))
                .then_with(|| left.stable_sprite_key.cmp(&right.stable_sprite_key))
                .then_with(|| right.size.total_cmp(&left.size))
        });
        bounds.sort_by_key(|bound| bound.entity);

        ParticleExtract {
            emitters,
            sprites,
            previous_sprites: Vec::new(),
            bounds,
            sort_camera_position: Some(camera_position),
            gpu_frame: gpu_frame_builder.finish(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ParticleGpuFrameContribution {
    frame: RenderParticleGpuFrameExtract,
    bounds: Option<RenderParticleBoundsSnapshot>,
}

#[derive(Default)]
struct ParticleGpuFrameBuilder {
    alive_count: u32,
    spawned_total: u32,
    per_emitter_spawned: Vec<u32>,
}

impl ParticleGpuFrameBuilder {
    fn push(&mut self, frame: RenderParticleGpuFrameExtract) {
        self.alive_count = self.alive_count.saturating_add(frame.alive_count);
        self.spawned_total = self.spawned_total.saturating_add(frame.spawned_total);
        self.per_emitter_spawned.extend(frame.per_emitter_spawned);
    }

    fn finish(self) -> Option<RenderParticleGpuFrameExtract> {
        let has_frame = self.alive_count > 0
            || self.spawned_total > 0
            || self.per_emitter_spawned.iter().any(|count| *count > 0);
        has_frame.then_some(RenderParticleGpuFrameExtract {
            alive_count: self.alive_count,
            spawned_total: self.spawned_total,
            per_emitter_spawned: self.per_emitter_spawned,
            indirect_draw_args: [6, self.alive_count, 0, 0],
        })
    }
}

fn collect_particle_sprites_from_value(
    entity: EntityId,
    render_layer_mask: u32,
    value: &serde_json::Value,
    output: &mut Vec<RenderParticleSpriteSnapshot>,
) {
    if let Some(entries) = value.get("sprites").and_then(serde_json::Value::as_array) {
        output.extend(
            entries
                .iter()
                .filter_map(|entry| particle_sprite(entity, render_layer_mask, entry)),
        );
        return;
    }
    if let Some(entries) = value.as_array() {
        output.extend(
            entries
                .iter()
                .filter_map(|entry| particle_sprite(entity, render_layer_mask, entry)),
        );
        return;
    }
    if let Some(sprite) = particle_sprite(entity, render_layer_mask, value) {
        output.push(sprite);
    }
}

fn particle_gpu_frame_contribution(
    value: &serde_json::Value,
) -> Option<ParticleGpuFrameContribution> {
    let frame_value = value.get("gpu_frame").unwrap_or(value);
    if !frame_value.is_object() {
        return None;
    }
    let mut per_emitter_spawned =
        u32_array_field(frame_value, "per_emitter_spawned").unwrap_or_default();
    let per_emitter_total = per_emitter_spawned
        .iter()
        .fold(0_u32, |total, count| total.saturating_add(*count));
    let alive_count = u32_field(frame_value, "alive_count")
        .or_else(|| u32_field(frame_value, "alive"))
        .unwrap_or(per_emitter_total);
    let spawned_total = u32_field(frame_value, "spawned_total")
        .or_else(|| u32_field(frame_value, "spawned"))
        .unwrap_or(per_emitter_total.max(alive_count));
    if per_emitter_spawned.is_empty() && (alive_count > 0 || spawned_total > 0) {
        per_emitter_spawned.push(spawned_total.max(alive_count));
    }
    if alive_count == 0 && spawned_total == 0 && per_emitter_spawned.iter().all(|count| *count == 0)
    {
        return None;
    }

    Some(ParticleGpuFrameContribution {
        frame: RenderParticleGpuFrameExtract {
            alive_count,
            spawned_total,
            per_emitter_spawned,
            indirect_draw_args: [6, alive_count, 0, 0],
        },
        bounds: particle_gpu_bounds(frame_value),
    })
}

fn particle_gpu_bounds(value: &serde_json::Value) -> Option<RenderParticleBoundsSnapshot> {
    let bounds = value.get("bounds").unwrap_or(value);
    let center = vec3_field(bounds, "center").or_else(|| vec3_field(value, "bounds_center"))?;
    let radius = positive_f32_field(bounds, "radius")
        .or_else(|| positive_f32_field(value, "bounds_radius"))?;
    Some(RenderParticleBoundsSnapshot {
        entity: 0,
        center,
        radius,
    })
}

fn particle_sprite(
    entity: EntityId,
    render_layer_mask: u32,
    value: &serde_json::Value,
) -> Option<RenderParticleSpriteSnapshot> {
    let position = vec3_field(value, "position")?;
    let size = f32_field(value, "size").filter(|size| *size > 0.0)?;
    let color = vec4_field(value, "color").unwrap_or(Vec4::new(1.0, 0.18, 0.12, 0.75));
    if color.w <= 0.0 {
        return None;
    }
    Some(RenderParticleSpriteSnapshot {
        entity,
        stable_sprite_key: stable_particle_sprite_key(value),
        position,
        size,
        aspect_ratio: positive_f32_field(value, "aspect_ratio").unwrap_or(1.0),
        billboard_offset: vec2_field(value, "billboard_offset").unwrap_or(Vec2::ZERO),
        rotation: f32_field(value, "rotation").unwrap_or(0.0),
        sort_order: i32_field(value, "sort_order").unwrap_or(0),
        color,
        intensity: f32_field(value, "intensity").unwrap_or(1.0).max(0.0),
        depth_test: true,
        render_layer_mask: RenderLayerSet::from_legacy_mask(render_layer_mask),
        material: None,
        texture: None,
    })
}

fn collect_world_hud_bar_sprites_from_value(
    entity: EntityId,
    render_layer_mask: u32,
    value: &serde_json::Value,
    output: &mut Vec<RenderParticleSpriteSnapshot>,
) {
    if let Some(entries) = value.get("bars").and_then(serde_json::Value::as_array) {
        for (bar_index, entry) in entries.iter().enumerate() {
            collect_world_hud_bar_sprites(entity, render_layer_mask, entry, bar_index, output);
        }
        return;
    }
    if let Some(entries) = value.as_array() {
        for (bar_index, entry) in entries.iter().enumerate() {
            collect_world_hud_bar_sprites(entity, render_layer_mask, entry, bar_index, output);
        }
        return;
    }
    collect_world_hud_bar_sprites(entity, render_layer_mask, value, 0, output);
}

fn collect_world_hud_bar_sprites(
    entity: EntityId,
    render_layer_mask: u32,
    value: &serde_json::Value,
    bar_index: usize,
    output: &mut Vec<RenderParticleSpriteSnapshot>,
) {
    if value
        .get("visible")
        .and_then(serde_json::Value::as_bool)
        .is_some_and(|visible| !visible)
    {
        return;
    }
    let Some(position) = vec3_field(value, "position") else {
        return;
    };
    let width = positive_f32_field(value, "width").unwrap_or(WORLD_HUD_DEFAULT_WIDTH);
    let height = positive_f32_field(value, "height").unwrap_or(WORLD_HUD_DEFAULT_HEIGHT);
    let ratio = world_hud_bar_ratio(value).unwrap_or(1.0).clamp(0.0, 1.0);
    let fill_color = vec4_field(value, "fill_color")
        .or_else(|| vec4_field(value, "color"))
        .unwrap_or(WORLD_HUD_FILL_COLOR);
    let back_color = vec4_field(value, "back_color")
        .or_else(|| vec4_field(value, "background_color"))
        .unwrap_or(WORLD_HUD_BACK_COLOR);
    let intensity = f32_field(value, "intensity").unwrap_or(1.0).max(0.0);
    let aspect_ratio = (width / height).max(1.0);
    let background_stable_sprite_key =
        world_hud_bar_stable_sprite_key(value, bar_index, WORLD_HUD_BAR_BACKGROUND_KEY_OFFSET);
    let fill_stable_sprite_key =
        world_hud_bar_stable_sprite_key(value, bar_index, WORLD_HUD_BAR_FILL_KEY_OFFSET);

    push_world_hud_bar_quad(
        entity,
        background_stable_sprite_key,
        position,
        height,
        aspect_ratio,
        Vec2::ZERO,
        back_color,
        intensity,
        WORLD_HUD_BACKGROUND_SORT_ORDER,
        render_layer_mask,
        output,
    );
    if ratio > 0.0 {
        let fill_width = width * ratio;
        let fill_offset = Vec2::new((fill_width - width) * 0.5, 0.0);
        push_world_hud_bar_quad(
            entity,
            fill_stable_sprite_key,
            position,
            height * 0.72,
            (fill_width / (height * 0.72)).max(1.0),
            fill_offset,
            fill_color,
            intensity,
            WORLD_HUD_FILL_SORT_ORDER,
            render_layer_mask,
            output,
        );
    }
}

fn push_world_hud_bar_quad(
    entity: EntityId,
    stable_sprite_key: u64,
    position: Vec3,
    size: f32,
    aspect_ratio: f32,
    billboard_offset: Vec2,
    color: Vec4,
    intensity: f32,
    sort_order: i32,
    render_layer_mask: u32,
    output: &mut Vec<RenderParticleSpriteSnapshot>,
) {
    if color.w <= 0.0 || size <= 0.0 {
        return;
    }
    output.push(RenderParticleSpriteSnapshot {
        entity,
        stable_sprite_key,
        position,
        size,
        aspect_ratio,
        billboard_offset,
        rotation: 0.0,
        sort_order,
        color,
        intensity,
        depth_test: false,
        render_layer_mask: RenderLayerSet::from_legacy_mask(render_layer_mask),
        material: None,
        texture: None,
    });
}

fn world_hud_bar_ratio(value: &serde_json::Value) -> Option<f32> {
    if let Some(ratio) = f32_field(value, "ratio") {
        return Some(ratio);
    }
    let current = f32_field(value, "value").or_else(|| f32_field(value, "hp"))?;
    let max = f32_field(value, "max").or_else(|| f32_field(value, "max_hp"))?;
    (max > 0.0).then_some(current / max)
}

fn world_hud_bar_stable_sprite_key(
    value: &serde_json::Value,
    bar_index: usize,
    part_offset: u64,
) -> u64 {
    let base_key = positive_u64_field(value, "stable_sprite_key")
        .or_else(|| positive_u64_field(value, "bar_key"))
        .or_else(|| positive_u64_field(value, "sprite_key"))
        .unwrap_or(bar_index as u64 + 1);
    base_key
        .saturating_sub(1)
        .saturating_mul(WORLD_HUD_BAR_KEY_STRIDE)
        .saturating_add(part_offset)
}

fn vec3_field(value: &serde_json::Value, field: &str) -> Option<Vec3> {
    let values = value.get(field)?.as_array()?;
    Some(Vec3::new(
        number_at(values, 0)?,
        number_at(values, 1)?,
        number_at(values, 2)?,
    ))
}

fn vec2_field(value: &serde_json::Value, field: &str) -> Option<Vec2> {
    let values = value.get(field)?.as_array()?;
    Some(Vec2::new(number_at(values, 0)?, number_at(values, 1)?))
}

fn vec4_field(value: &serde_json::Value, field: &str) -> Option<Vec4> {
    let values = value.get(field)?.as_array()?;
    Some(Vec4::new(
        number_at(values, 0)?,
        number_at(values, 1)?,
        number_at(values, 2)?,
        number_at(values, 3)?,
    ))
}

fn f32_field(value: &serde_json::Value, field: &str) -> Option<f32> {
    value.get(field)?.as_f64().map(|value| value as f32)
}

fn i32_field(value: &serde_json::Value, field: &str) -> Option<i32> {
    value
        .get(field)?
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
}

fn u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

fn u32_field(value: &serde_json::Value, field: &str) -> Option<u32> {
    value
        .get(field)?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

fn u32_array_field(value: &serde_json::Value, field: &str) -> Option<Vec<u32>> {
    let values = value.get(field)?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
            .collect(),
    )
}

fn positive_u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    u64_field(value, field).filter(|value| *value > 0)
}

fn stable_particle_sprite_key(value: &serde_json::Value) -> u64 {
    u64_field(value, "stable_sprite_key")
        .or_else(|| u64_field(value, "sprite_key"))
        .unwrap_or(0)
}

fn positive_f32_field(value: &serde_json::Value, field: &str) -> Option<f32> {
    f32_field(value, field).filter(|value| value.is_finite() && *value > 0.0)
}

fn number_at(values: &[serde_json::Value], index: usize) -> Option<f32> {
    values.get(index)?.as_f64().map(|value| value as f32)
}

fn distance_sort_key(position: Vec3, camera_position: Vec3) -> f32 {
    (position - camera_position).length_squared()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn world_hud_bar_sprites_use_nonzero_stable_keys() {
        let mut sprites = Vec::new();

        collect_world_hud_bar_sprites_from_value(
            42,
            1 << 4,
            &json!({
                "position": [0.0, 1.0, 2.0],
                "ratio": 0.5
            }),
            &mut sprites,
        );

        assert_eq!(stable_sprite_keys(&sprites), vec![1, 2]);
        assert!(sprites
            .iter()
            .all(|sprite| sprite.render_layer_mask.to_legacy_mask_lossy() == 1 << 4));
    }

    #[test]
    fn world_hud_bar_array_sprites_use_bar_indexed_stable_keys() {
        let mut sprites = Vec::new();

        collect_world_hud_bar_sprites_from_value(
            42,
            u32::MAX,
            &json!({
                "bars": [
                    { "position": [0.0, 1.0, 2.0], "ratio": 0.5 },
                    { "position": [0.0, 2.0, 2.0], "ratio": 1.0 }
                ]
            }),
            &mut sprites,
        );

        assert_eq!(stable_sprite_keys(&sprites), vec![1, 2, 3, 4]);
    }

    #[test]
    fn world_hud_bar_sprites_use_overlay_depth_path() {
        let mut sprites = Vec::new();

        collect_world_hud_bar_sprites_from_value(
            42,
            u32::MAX,
            &json!({
                "position": [0.0, 1.0, 2.0],
                "ratio": 0.5
            }),
            &mut sprites,
        );

        assert!(!sprites.is_empty());
        assert!(sprites.iter().all(|sprite| !sprite.depth_test));
    }

    #[test]
    fn authored_particle_sprites_keep_depth_test_path() {
        let sprite = particle_sprite(
            42,
            1 << 5,
            &json!({
                "position": [0.0, 1.0, 2.0],
                "size": 0.25
            }),
        )
        .expect("valid authored particle sprite");

        assert!(sprite.depth_test);
        assert_eq!(sprite.render_layer_mask.to_legacy_mask_lossy(), 1 << 5);
    }

    #[test]
    fn particle_gpu_frame_contribution_defaults_indirect_args_to_alive_count() {
        let contribution = particle_gpu_frame_contribution(&json!({
            "gpu_frame": {
                "alive_count": 5,
                "spawned_total": 7,
                "per_emitter_spawned": [2, 5],
                "bounds": {
                    "center": [1.0, 2.0, 3.0],
                    "radius": 4.0
                }
            }
        }))
        .expect("gpu frame should parse");

        assert_eq!(contribution.frame.alive_count, 5);
        assert_eq!(contribution.frame.spawned_total, 7);
        assert_eq!(contribution.frame.per_emitter_spawned, vec![2, 5]);
        assert_eq!(contribution.frame.indirect_draw_args, [6, 5, 0, 0]);
        assert_eq!(
            contribution.bounds,
            Some(RenderParticleBoundsSnapshot {
                entity: 0,
                center: Vec3::new(1.0, 2.0, 3.0),
                radius: 4.0
            })
        );
    }

    #[test]
    fn particle_gpu_frame_builder_aggregates_scene_visible_emitters() {
        let mut builder = ParticleGpuFrameBuilder::default();
        builder.push(RenderParticleGpuFrameExtract {
            alive_count: 2,
            spawned_total: 3,
            per_emitter_spawned: vec![3],
            indirect_draw_args: [6, 2, 0, 0],
        });
        builder.push(RenderParticleGpuFrameExtract {
            alive_count: 4,
            spawned_total: 5,
            per_emitter_spawned: vec![2, 3],
            indirect_draw_args: [6, 4, 0, 0],
        });

        let frame = builder.finish().expect("aggregate gpu frame");

        assert_eq!(frame.alive_count, 6);
        assert_eq!(frame.spawned_total, 8);
        assert_eq!(frame.per_emitter_spawned, vec![3, 2, 3]);
        assert_eq!(frame.indirect_draw_args, [6, 6, 0, 0]);
    }

    fn stable_sprite_keys(sprites: &[RenderParticleSpriteSnapshot]) -> Vec<u64> {
        sprites
            .iter()
            .map(|sprite| sprite.stable_sprite_key)
            .collect()
    }
}
