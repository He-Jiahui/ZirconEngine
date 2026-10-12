use super::*;

fn canonical_values(count: usize) -> BTreeMap<String, Value> {
    (0..count)
        .map(|index| {
            (
                format!("editor.component_{index:04}.metric.value"),
                Value::Integer(index as i64),
            )
        })
        .collect()
}

#[test]
fn borrowed_custom_property_aliases_preserve_cloned_projection() {
    let values = BTreeMap::from([
        (
            "editor.accent".to_string(),
            Value::String("#fff".to_string()),
        ),
        ("editor.density.row.height".to_string(), Value::Float(24.0)),
        ("editor.surface.0".to_string(), Value::Integer(0)),
    ]);

    assert_eq!(
        custom_property_aliases(&values),
        custom_property_aliases_cloning(&values),
    );
}

#[test]
fn runtime_interface03_batch72_73_single_buffer_alias_names_preserve_replacing_projection() {
    for canonical_name in [
        "editor.accent",
        "editor.density.row.height",
        "editor..surface.0",
        ".editor.trailing.",
        "editor.字体.正文",
        "plain",
        "",
    ] {
        assert_eq!(
            custom_property_alias_name(canonical_name),
            custom_property_alias_name_replacing(canonical_name),
        );
    }
}

#[test]
#[ignore = "release-only single-buffer cascade alias name benchmark"]
fn runtime_interface03_batch72_73_single_buffer_cascade_alias_name_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const TOKEN_COUNT: usize = 4_096;
    const LOOKUP_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 11;
    let values = canonical_values(TOKEN_COUNT);
    let canonical_names = values.keys().map(String::as_str).collect::<Vec<_>>();
    let mut replacing_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_buffer_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_replacing = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                for canonical_name in &canonical_names {
                    black_box(custom_property_alias_name_replacing(black_box(
                        canonical_name,
                    )));
                }
            }
            started.elapsed().as_nanos()
        };
        let measure_single_buffer = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                for canonical_name in &canonical_names {
                    black_box(custom_property_alias_name(black_box(canonical_name)));
                }
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            replacing_samples.push(measure_replacing());
            single_buffer_samples.push(measure_single_buffer());
        } else {
            single_buffer_samples.push(measure_single_buffer());
            replacing_samples.push(measure_replacing());
        }
    }

    replacing_samples.sort_unstable();
    single_buffer_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_BUFFER_CASCADE_ALIAS_NAME_BENCH_V1 tokens={TOKEN_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} replacing_p95_ns={} single_buffer_p95_ns={}",
        replacing_samples[p95], single_buffer_samples[p95],
    );
    assert!(
        single_buffer_samples[p95].saturating_mul(5) <= replacing_samples[p95].saturating_mul(4),
        "single-buffer alias names must improve P95 by at least 20%: replacing={}ns single_buffer={}ns",
        replacing_samples[p95],
        single_buffer_samples[p95],
    );
}

#[test]
#[ignore = "release-only borrowed cascade alias projection benchmark"]
fn runtime_interface03_batch35_borrowed_cascade_alias_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const TOKEN_COUNT: usize = 4_096;
    const LOOKUP_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 11;
    let values = canonical_values(TOKEN_COUNT);
    let mut cloning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloning = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(custom_property_aliases_cloning(black_box(&values)));
            }
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(custom_property_aliases(black_box(&values)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloning_samples.push(measure_cloning());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            cloning_samples.push(measure_cloning());
        }
    }

    cloning_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_CASCADE_ALIAS_BENCH_V1 tokens={TOKEN_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} cloning_p95_ns={} borrowed_p95_ns={}",
        cloning_samples[p95], borrowed_samples[p95],
    );
    assert!(
        borrowed_samples[p95].saturating_mul(5) <= cloning_samples[p95].saturating_mul(4),
        "borrowed cascade aliases must improve P95 by at least 20%: cloning={}ns borrowed={}ns",
        cloning_samples[p95],
        borrowed_samples[p95],
    );
}
