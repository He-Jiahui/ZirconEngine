struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) local_position: vec2<f32>,
    @location(3) half_extent: vec2<f32>,
    @location(4) corner_radius: f32,
    @location(5) border_width: f32,
    @location(6) fill_color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) @interpolate(flat) half_extent: vec2<f32>,
    @location(3) @interpolate(flat) corner_radius: f32,
    @location(4) @interpolate(flat) border_width: f32,
    @location(5) @interpolate(flat) fill_color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.color = input.color;
    out.local_position = input.local_position;
    out.half_extent = input.half_extent;
    out.corner_radius = input.corner_radius;
    out.border_width = input.border_width;
    out.fill_color = input.fill_color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    if input.border_width > 0.0 && input.fill_color.a > 0.0 {
        let coverages = rounded_box_coverages(
            input.local_position,
            input.half_extent,
            input.corner_radius,
            input.border_width,
        );
        let inner_coverage = min(coverages.y, coverages.x);
        let border_coverage = max(coverages.x - inner_coverage, 0.0);
        let fill_alpha = input.fill_color.a * inner_coverage;
        let border_alpha = input.color.a * border_coverage;
        let alpha = fill_alpha + border_alpha;
        if alpha <= 0.0 {
            return vec4<f32>(0.0);
        }
        let premultiplied_rgb =
            input.fill_color.rgb * fill_alpha + input.color.rgb * border_alpha;
        return vec4<f32>(premultiplied_rgb / alpha, alpha);
    }
    let coverage = rounded_box_coverage(
        input.local_position,
        input.half_extent,
        input.corner_radius,
        input.border_width,
    );
    return vec4<f32>(input.color.rgb, input.color.a * coverage);
}

fn rounded_box_distance(local_position: vec2<f32>, half_extent: vec2<f32>, radius: f32) -> f32 {
    let q = abs(local_position) - half_extent + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn rounded_box_alpha(signed_distance: f32, distance_width: f32) -> f32 {
    return 1.0 - smoothstep(
        distance_width * -0.5,
        distance_width * 0.5,
        signed_distance,
    );
}

fn rounded_box_coverages(
    local_position: vec2<f32>,
    half_extent: vec2<f32>,
    corner_radius: f32,
    border_width: f32,
) -> vec2<f32> {
    // Integrate a 4x4 local grid in each physical target pixel. Each sub-sample
    // keeps a quarter-pixel analytic filter so added detail does not widen edges.
    let pixel_step = vec2<f32>(
        max(fwidth(local_position.x), 0.0001),
        max(fwidth(local_position.y), 0.0001),
    );
    let inner_half_extent = half_extent - vec2<f32>(border_width);
    let has_inner_edge = border_width > 0.0 && all(inner_half_extent > vec2<f32>(0.0));
    let outer_distance = rounded_box_distance(local_position, half_extent, corner_radius);
    let distance_width = max(fwidth(outer_distance), 0.0001);
    let coverage_guard = distance_width * 0.75;
    let safe_inner_half_extent = max(inner_half_extent, vec2<f32>(0.0001));
    let inner_distance = rounded_box_distance(
        local_position,
        safe_inner_half_extent,
        max(corner_radius - border_width, 0.0),
    );
    let inner_distance_width = max(fwidth(inner_distance), 0.0001);
    let inner_coverage_guard = inner_distance_width * 0.75;

    if !has_inner_edge {
        if outer_distance <= coverage_guard * -1.0 {
            return vec2<f32>(1.0, 0.0);
        }
        if outer_distance >= coverage_guard {
            return vec2<f32>(0.0, 0.0);
        }
    } else {
        if outer_distance >= coverage_guard {
            return vec2<f32>(0.0, 0.0);
        }
        if inner_distance <= inner_coverage_guard * -1.0 {
            return vec2<f32>(1.0, 1.0);
        }
        if outer_distance <= coverage_guard * -1.0
            && inner_distance >= inner_coverage_guard
        {
            return vec2<f32>(1.0, 0.0);
        }
    }

    let subpixel_filter_scale = 0.25;
    let sample_offsets = array<vec2<f32>, 16>(
        vec2<f32>(-0.375, -0.375),
        vec2<f32>(-0.125, -0.375),
        vec2<f32>(0.125, -0.375),
        vec2<f32>(0.375, -0.375),
        vec2<f32>(-0.375, -0.125),
        vec2<f32>(-0.125, -0.125),
        vec2<f32>(0.125, -0.125),
        vec2<f32>(0.375, -0.125),
        vec2<f32>(-0.375, 0.125),
        vec2<f32>(-0.125, 0.125),
        vec2<f32>(0.125, 0.125),
        vec2<f32>(0.375, 0.125),
        vec2<f32>(-0.375, 0.375),
        vec2<f32>(-0.125, 0.375),
        vec2<f32>(0.125, 0.375),
        vec2<f32>(0.375, 0.375),
    );
    var outer_coverage_sum = 0.0;
    var inner_coverage_sum = 0.0;
    for (var sample_index = 0u; sample_index < 16u; sample_index++) {
        let sample_position = local_position + sample_offsets[sample_index] * pixel_step;
        let sample_outer_distance = rounded_box_distance(
            sample_position,
            half_extent,
            corner_radius,
        );
        let outer_coverage = rounded_box_alpha(
            sample_outer_distance,
            distance_width * subpixel_filter_scale,
        );
        var inner_coverage = 0.0;
        if has_inner_edge {
            let sample_inner_distance = rounded_box_distance(
                sample_position,
                inner_half_extent,
                max(corner_radius - border_width, 0.0),
            );
            inner_coverage = rounded_box_alpha(
                sample_inner_distance,
                inner_distance_width * subpixel_filter_scale,
            );
        }
        outer_coverage_sum += outer_coverage;
        inner_coverage_sum += inner_coverage;
    }
    return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625;
}

fn rounded_box_coverage(
    local_position: vec2<f32>,
    half_extent: vec2<f32>,
    corner_radius: f32,
    border_width: f32,
) -> f32 {
    let coverages = rounded_box_coverages(
        local_position,
        half_extent,
        corner_radius,
        border_width,
    );
    if border_width > 0.0 {
        return max(coverages.x - coverages.y, 0.0);
    }
    return coverages.x;
}
