@group(0) @binding(0)
var scene_depth_tex: texture_depth_2d;

@group(0) @binding(1)
var<storage, read_write> hybrid_gi_scene_words: array<u32>;

const HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x48474944u;
const DEPTH_Q24_SCALE: f32 = 16777215.0;

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x != 0u || global_id.y != 0u || global_id.z != 0u) {
        return;
    }

    let dimensions = textureDimensions(scene_depth_tex);
    let center = vec2<i32>(
        i32(dimensions.x / 2u),
        i32(dimensions.y / 2u),
    );
    let depth = clamp(textureLoad(scene_depth_tex, center, 0), 0.0, 1.0);

    hybrid_gi_scene_words[0] = HYBRID_GI_SCENE_DEPTH_HANDOFF_MAGIC;
    hybrid_gi_scene_words[1] = dimensions.x;
    hybrid_gi_scene_words[2] = dimensions.y;
    hybrid_gi_scene_words[3] = u32((depth * DEPTH_Q24_SCALE) + 0.5);
    hybrid_gi_scene_words[4] = 1u;
}
