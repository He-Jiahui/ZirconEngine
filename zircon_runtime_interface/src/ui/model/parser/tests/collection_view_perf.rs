use super::*;

fn collection_view_arguments() -> Vec<UiBindingValue> {
    vec![
        UiBindingValue::String("provider.primary".to_string()),
        UiBindingValue::Unsigned(7),
        UiBindingValue::String("schema.item".to_string()),
        UiBindingValue::Unsigned(11),
        UiBindingValue::Unsigned(19),
        UiBindingValue::Unsigned(23),
        UiBindingValue::Unsigned(29),
        UiBindingValue::Unsigned(31),
    ]
}

fn construct_collection_view_removing(
    constructor: &str,
    mut arguments: Vec<UiBindingValue>,
) -> Result<UiBindingValue, UiBindingParseError> {
    require_arity(constructor, &arguments, 8)?;
    let total_length = unsigned_argument(constructor, &arguments, 7)?;
    let length = unsigned_argument(constructor, &arguments, 6)?
        .try_into()
        .map_err(|_| invalid_constructor(constructor, "window length exceeds u32"))?;
    let offset = unsigned_argument(constructor, &arguments, 5)?;
    let revision = unsigned_argument(constructor, &arguments, 4)?;
    let item_schema_version =
        UiModelSchemaVersion::try_new(unsigned_argument(constructor, &arguments, 3)?)
            .map_err(|error| invalid_constructor(constructor, &error.to_string()))?;
    let item_schema_id = take_string_argument(constructor, arguments.remove(2), 2)?;
    let provider_version =
        UiModelProviderVersion::try_new(unsigned_argument(constructor, &arguments, 1)?)
            .map_err(|error| invalid_constructor(constructor, &error.to_string()))?;
    let provider_id = take_string_argument(constructor, arguments.remove(0), 0)?;
    let provider = UiModelProviderKey {
        id: UiModelProviderId::try_new(provider_id)
            .map_err(|error| invalid_constructor(constructor, &error.to_string()))?,
        version: provider_version,
    };
    let item_schema = UiModelSchemaKey {
        id: UiModelSchemaId::try_new(item_schema_id)
            .map_err(|error| invalid_constructor(constructor, &error.to_string()))?,
        version: item_schema_version,
    };
    Ok(UiBindingValue::CollectionView(
        UiBindingCollectionView::try_new(
            provider,
            item_schema,
            revision,
            offset,
            length,
            total_length,
        )?,
    ))
}

#[test]
fn runtime_interface03_batch44_46_moved_collection_view_arguments_preserve_remove_results_and_error_order(
) {
    let valid = collection_view_arguments();
    assert_eq!(
        BindingParser::new("").construct_collection_view("collection_view", valid.clone()),
        construct_collection_view_removing("collection_view", valid.clone()),
    );

    for index in 0..valid.len() {
        let mut invalid = valid.clone();
        invalid[index] = UiBindingValue::Null;
        assert_eq!(
            BindingParser::new("").construct_collection_view("collection_view", invalid.clone()),
            construct_collection_view_removing("collection_view", invalid),
            "argument {index} must preserve the existing failure",
        );
    }

    for index in [1, 3] {
        let mut zero_version = valid.clone();
        zero_version[index] = UiBindingValue::Unsigned(0);
        assert_eq!(
            BindingParser::new("")
                .construct_collection_view("collection_view", zero_version.clone(),),
            construct_collection_view_removing("collection_view", zero_version),
            "version argument {index} must preserve the existing failure",
        );
    }

    for invalid_indexes in [[2, 1], [1, 0], [3, 2]] {
        let mut invalid = valid.clone();
        for index in invalid_indexes {
            invalid[index] = UiBindingValue::Null;
        }
        assert_eq!(
            BindingParser::new("").construct_collection_view("collection_view", invalid.clone()),
            construct_collection_view_removing("collection_view", invalid),
            "multiple failures must preserve the existing argument-check order",
        );
    }

    for wrong_arity in [valid[..7].to_vec(), {
        let mut values = valid.clone();
        values.push(UiBindingValue::Null);
        values
    }] {
        assert_eq!(
            BindingParser::new("")
                .construct_collection_view("collection_view", wrong_arity.clone()),
            construct_collection_view_removing("collection_view", wrong_arity),
        );
    }
}

fn move_arguments_removing(mut arguments: Vec<UiBindingValue>) -> (UiBindingValue, UiBindingValue) {
    let item_schema_id = arguments.remove(2);
    let provider_id = arguments.remove(0);
    (provider_id, item_schema_id)
}

fn move_arguments_consuming(arguments: Vec<UiBindingValue>) -> (UiBindingValue, UiBindingValue) {
    let mut arguments = arguments.into_iter();
    let provider_id = arguments.next().unwrap();
    let _ = arguments.next();
    let item_schema_id = arguments.next().unwrap();
    (provider_id, item_schema_id)
}

fn benchmark_argument_batches(count: usize) -> Vec<Vec<UiBindingValue>> {
    (0..count)
        .map(|_| {
            vec![
                UiBindingValue::Unsigned(0),
                UiBindingValue::Unsigned(1),
                UiBindingValue::Unsigned(2),
                UiBindingValue::Unsigned(3),
                UiBindingValue::Unsigned(4),
                UiBindingValue::Unsigned(5),
                UiBindingValue::Unsigned(6),
                UiBindingValue::Unsigned(7),
            ]
        })
        .collect()
}

#[test]
#[ignore = "release-only moved collection-view argument benchmark"]
fn runtime_interface03_batch44_46_moved_collection_view_arguments_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const MOVE_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 11;
    let mut removing_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fixed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let removing_inputs = benchmark_argument_batches(MOVE_COUNT);
        let fixed_inputs = benchmark_argument_batches(MOVE_COUNT);
        let measure_removing = move || {
            let started = Instant::now();
            for arguments in removing_inputs {
                black_box(move_arguments_removing(black_box(arguments)));
            }
            started.elapsed().as_nanos()
        };
        let measure_fixed = move || {
            let started = Instant::now();
            for arguments in fixed_inputs {
                black_box(move_arguments_consuming(black_box(arguments)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            removing_samples.push(measure_removing());
            fixed_samples.push(measure_fixed());
        } else {
            fixed_samples.push(measure_fixed());
            removing_samples.push(measure_removing());
        }
    }

    removing_samples.sort_unstable();
    fixed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MOVED_COLLECTION_VIEW_ARGUMENTS_BENCH_V1 moves={MOVE_COUNT} samples={SAMPLE_COUNT} removing_p95_ns={} fixed_p95_ns={}",
        removing_samples[p95], fixed_samples[p95],
    );
    assert!(
        fixed_samples[p95].saturating_mul(5) <= removing_samples[p95].saturating_mul(4),
        "fixed collection-view argument moves must improve P95 by at least 20%: removing={}ns fixed={}ns",
        removing_samples[p95],
        fixed_samples[p95],
    );
}
