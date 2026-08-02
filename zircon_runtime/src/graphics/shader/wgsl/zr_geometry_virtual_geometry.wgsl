const ZR_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX: u32 = 4u;
const ZR_VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT: u32 = 16u;

fn zr_virtual_geometry_payload_slot(instance_index: u32) -> u32 {
    return zr_gpu_scene_primitive_for_instance(instance_index).payload_slot;
}

fn zr_virtual_geometry_vertex_word_index(instance_index: u32, vertex_ordinal: u32) -> u32 {
    let payload_slot = zr_virtual_geometry_payload_slot(instance_index);
    if (!zr_gpu_scene_valid_payload_slot(payload_slot, zr_gpu_scene_virtual_geometry_page_count())) {
        return 0xffffffffu;
    }
    let page = zr_virtual_geometry_pages[payload_slot];
    let cluster_base = page.x;
    let vertex_count = page.y;
    if (vertex_ordinal >= vertex_count) {
        return 0xffffffffu;
    }
    return cluster_base + vertex_ordinal * ZR_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX;
}

fn zr_virtual_geometry_has_vertex(instance_index: u32, vertex_ordinal: u32) -> bool {
    let word_index = zr_virtual_geometry_vertex_word_index(instance_index, vertex_ordinal);
    let cluster_word_count = zr_gpu_scene_virtual_geometry_cluster_word_count();
    return word_index != 0xffffffffu
        && cluster_word_count >= ZR_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX
        && word_index <= cluster_word_count - ZR_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX;
}

fn zr_virtual_geometry_cluster_word(
    instance_index: u32,
    vertex_ordinal: u32,
    word_offset: u32,
) -> vec4<f32> {
    let word_index = zr_virtual_geometry_vertex_word_index(instance_index, vertex_ordinal);
    return zr_virtual_geometry_clusters[word_index + word_offset];
}

fn zr_virtual_geometry_vertex_ordinal(v: ZrVertexInput) -> u32 {
    return v.joints.x | (v.joints.y << ZR_VIRTUAL_GEOMETRY_VERTEX_ORDINAL_HIGH_SHIFT);
}

fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    let vertex_ordinal = zr_virtual_geometry_vertex_ordinal(v);
    if (!zr_virtual_geometry_has_vertex(instance_index, vertex_ordinal)) {
        return v.position;
    }
    return zr_virtual_geometry_cluster_word(instance_index, vertex_ordinal, 0u).xyz;
}

fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    return fetch_position(v, instance_index);
}

fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32> {
    let vertex_ordinal = zr_virtual_geometry_vertex_ordinal(v);
    if (!zr_virtual_geometry_has_vertex(instance_index, vertex_ordinal)) {
        return v.normal;
    }
    return zr_normalize_or_zero(zr_virtual_geometry_cluster_word(instance_index, vertex_ordinal, 1u).xyz);
}

fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    let vertex_ordinal = zr_virtual_geometry_vertex_ordinal(v);
    if (!zr_virtual_geometry_has_vertex(instance_index, vertex_ordinal)) {
        return v.tangent;
    }
    return zr_virtual_geometry_cluster_word(instance_index, vertex_ordinal, 2u);
}

fn fetch_uv0(v: ZrVertexInput) -> vec2<f32> {
    return v.uv0;
}

fn fetch_uv1(v: ZrVertexInput) -> vec2<f32> {
    return v.uv1;
}

fn fetch_color(v: ZrVertexInput, instance_index: u32) -> vec4<f32> {
    _ = instance_index;
    return v.color;
}
