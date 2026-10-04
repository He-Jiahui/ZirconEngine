use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
struct BenchmarkDocument {
    asset_id: String,
    nodes: Vec<String>,
    tokens: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BenchmarkSource {
    path: PathBuf,
    document: BenchmarkDocument,
}

#[derive(Debug, PartialEq, Eq)]
struct BenchmarkCacheEntry {
    root_document: BenchmarkDocument,
    imported_documents: Vec<BenchmarkDocument>,
    source_paths: Vec<PathBuf>,
}

#[test]
fn optimization_batch_hm_runtime589_file_cache_moves_owned_sources() {
    let source = include_str!("../file_cache.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let build = production
        .split("fn build_file_store_cache_entry")
        .nth(1)
        .expect("file store cache entry builder")
        .split("fn resolve_resource_reference_path")
        .next()
        .expect("builder ends before reference resolution");

    assert!(build.contains("let mut sources = sources.into_iter();"));
    assert!(build.contains("let mut root_document = root_source.document;"));
    assert!(build.contains("source_paths.push(source.path);"));
    assert!(!build.contains("source.document.clone()"));
    assert!(!build.contains("source.path.clone()"));
    assert!(!production.contains("fn root_document_with_imported_styles"));
}

fn benchmark_sources(source_count: usize, root_node_count: usize) -> Vec<BenchmarkSource> {
    (0..source_count)
        .map(|source_index| BenchmarkSource {
            path: PathBuf::from(format!(
                "C:/runtime589/assets/ui/source-{source_index:04}.zui"
            )),
            document: BenchmarkDocument {
                asset_id: format!("runtime589.document.{source_index:04}"),
                nodes: (0..if source_index == 0 {
                    root_node_count
                } else {
                    32
                })
                    .map(|node_index| {
                        format!(
                            "source-{source_index:04}-node-{node_index:05}-{}",
                            "retained-node-payload".repeat(3)
                        )
                    })
                    .collect(),
                tokens: (0..16)
                    .map(|token_index| format!("token-{source_index:04}-{token_index:03}"))
                    .collect(),
            },
        })
        .collect()
}

fn legacy_build(sources: Vec<BenchmarkSource>) -> BenchmarkCacheEntry {
    let mut root_document = sources[0].document.clone();
    for source in sources.iter().skip(1) {
        root_document.tokens.extend(source.document.tokens.clone());
    }
    let source_paths = sources.iter().map(|source| source.path.clone()).collect();
    let imported_documents = sources
        .into_iter()
        .skip(1)
        .map(|source| source.document)
        .collect();
    BenchmarkCacheEntry {
        root_document,
        imported_documents,
        source_paths,
    }
}

fn moved_build(sources: Vec<BenchmarkSource>) -> BenchmarkCacheEntry {
    let mut sources = sources.into_iter();
    let root_source = sources.next().expect("benchmark root source");
    let mut root_document = root_source.document;
    let mut imported_documents = Vec::with_capacity(sources.len());
    let mut source_paths = Vec::with_capacity(sources.len() + 1);
    source_paths.push(root_source.path);
    for source in sources {
        root_document.tokens.extend(source.document.tokens.clone());
        imported_documents.push(source.document);
        source_paths.push(source.path);
    }
    BenchmarkCacheEntry {
        root_document,
        imported_documents,
        source_paths,
    }
}

fn measure_build(
    sources: Vec<BenchmarkSource>,
    build: fn(Vec<BenchmarkSource>) -> BenchmarkCacheEntry,
) -> u128 {
    let started = Instant::now();
    let entry = build(sources);
    black_box(&entry);
    let elapsed = started.elapsed().as_nanos().max(1);
    drop(entry);
    elapsed
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hm_runtime589_file_cache_source_move_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const SOURCE_COUNT: usize = 256;
    const ROOT_NODE_COUNT: usize = 8_192;

    let seed = benchmark_sources(SOURCE_COUNT, ROOT_NODE_COUNT);
    assert_eq!(moved_build(seed.clone()), legacy_build(seed.clone()));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_input = seed.clone();
        let optimized_input = seed.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_build(legacy_input, legacy_build));
            optimized_samples.push(measure_build(optimized_input, moved_build));
        } else {
            optimized_samples.push(measure_build(optimized_input, moved_build));
            legacy_samples.push(measure_build(legacy_input, legacy_build));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME589_UI_V2_CACHE_SOURCE_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             sources_per_sample={SOURCE_COUNT} root_nodes={ROOT_NODE_COUNT} \
             legacy_root_document_clones=1 optimized_root_document_clones=0 \
             legacy_path_clones={SOURCE_COUNT} optimized_path_clones=0 \
             legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 60,
        "moved source P95 {optimized_p95}ns exceeded 60% of cloned P95 {legacy_p95}ns"
    );
}
