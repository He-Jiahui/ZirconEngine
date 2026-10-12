use super::super::UiSurfaceRect;
use super::*;

#[test]
fn optimization_batch_20260830cr_runtime505_style_tables_reserve_command_upper_bound() {
    let source = include_str!("../compact_styles.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("UI surface style production source");

    assert!(production.contains("let style_capacity = commands.len();"));
    assert!(production.contains("Vec::with_capacity(style_capacity)"));
    assert!(production.contains(
        "HashMap::<UiSurfaceStyleKey, UiSurfaceStyleHandle>::with_capacity(style_capacity)"
    ));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cr_runtime505_style_table_capacity_evidence() {
    const COMMAND_COUNT: usize = 32_768;
    const MARKER: &str = "RUNTIME505_UI_SURFACE_STYLE_CAPACITY_BENCH_V1";
    let (legacy_vec_growth_events, legacy_map_growth_events) =
        style_table_growth_events(COMMAND_COUNT, false);
    let (optimized_vec_growth_events, optimized_map_growth_events) =
        style_table_growth_events(COMMAND_COUNT, true);
    let commands = (0..COMMAND_COUNT).map(unique_quad_command).collect();

    let started = std::time::Instant::now();
    let (commands, styles) = compact_commands(commands);
    let elapsed = started.elapsed();

    assert_eq!(commands.len(), COMMAND_COUNT);
    assert_eq!(styles.len(), COMMAND_COUNT);
    assert!(legacy_vec_growth_events > 0);
    assert!(legacy_map_growth_events > 0);
    assert_eq!(optimized_vec_growth_events, 0);
    assert_eq!(optimized_map_growth_events, 0);
    println!(
        "{MARKER} commands={COMMAND_COUNT} legacy_vec_growth_events={legacy_vec_growth_events} legacy_map_growth_events={legacy_map_growth_events} optimized_vec_growth_events={optimized_vec_growth_events} optimized_map_growth_events={optimized_map_growth_events} reduction_pct=100 elapsed_micros={}",
        elapsed.as_micros()
    );
}

fn unique_quad_command(index: usize) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index: index as i32,
        frame: UiSurfaceRect::new(0.0, 0.0, 1.0, 1.0),
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color: (index as u32).to_le_bytes(),
            corner_radius: index as f32,
        },
    }
}

fn style_table_growth_events(count: usize, reserve: bool) -> (usize, usize) {
    let mut styles = if reserve {
        Vec::with_capacity(count)
    } else {
        Vec::new()
    };
    let mut handles = if reserve {
        HashMap::with_capacity(count)
    } else {
        HashMap::new()
    };
    let mut vec_growth_events = 0;
    let mut map_growth_events = 0;
    for index in 0..count {
        let previous_vec_capacity = styles.capacity();
        styles.push(index);
        vec_growth_events += usize::from(styles.capacity() != previous_vec_capacity);

        let previous_map_capacity = handles.capacity();
        handles.insert(index, index);
        map_growth_events += usize::from(handles.capacity() != previous_map_capacity);
    }
    (vec_growth_events, map_growth_events)
}
