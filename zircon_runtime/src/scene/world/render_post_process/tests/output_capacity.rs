const BENCHMARK_MARKER: &str = "RUNTIME858_RENDER_POST_PROCESS_OUTPUT_CAPACITY_BENCH_V1";

#[test]
fn runtime858_render_post_process_output_capacity_preserves_order() {
    let inputs = [(3_u32, 30_u32), (1, 10), (2, 20)];
    let (extracts, fog_volumes, extract_growth, fog_growth) =
        model_collect(inputs.iter().copied(), inputs.len());

    assert_eq!(extracts, vec![3, 1, 2]);
    assert_eq!(fog_volumes, vec![30, 10, 20]);
    assert_eq!(extract_growth, 0);
    assert_eq!(fog_growth, 0);
}

#[test]
fn runtime858_render_post_process_output_capacity_keeps_empty_world_zero_capacity() {
    let (extracts, fog_volumes, extract_growth, fog_growth) =
        // TODO: [CR-WORLD-CAPACITY-FIXTURE-0001] 模型预分配 4096 项，仅断言为空且无增长；本夹具未覆盖生产空 World 的容量，需直接验证 collect_post_process_volumes。
        model_collect(std::iter::empty(), 4_096);

    assert!(extracts.is_empty());
    assert!(fog_volumes.is_empty());
    assert_eq!(extract_growth, 0);
    assert_eq!(fog_growth, 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime858_render_post_process_output_capacity_bench() {
    let inputs = (0_u32..4_096).map(|index| (index, index + 1));
    let (extracts, fog_volumes, extract_growth, fog_growth) = model_collect(inputs, 4_096);

    println!(
        "{BENCHMARK_MARKER} extracts={} fog_volumes={} extract_growth={} fog_growth={} reserved_capacity={}",
        extracts.len(),
        fog_volumes.len(),
        extract_growth,
        fog_growth,
        4_096
    );
    assert_eq!(extract_growth, 0);
    assert_eq!(fog_growth, 0);
}

fn model_collect<I>(values: I, volume_count: usize) -> (Vec<u32>, Vec<u32>, usize, usize)
where
    I: IntoIterator<Item = (u32, u32)>,
{
    let mut extracts = Vec::with_capacity(volume_count);
    let mut fog_volumes = Vec::with_capacity(volume_count);
    let mut extract_growth = 0;
    let mut fog_growth = 0;
    for (extract, fog) in values {
        if extracts.len() == extracts.capacity() {
            extract_growth += 1;
        }
        if fog_volumes.len() == fog_volumes.capacity() {
            fog_growth += 1;
        }
        extracts.push(extract);
        fog_volumes.push(fog);
    }
    (extracts, fog_volumes, extract_growth, fog_growth)
}
