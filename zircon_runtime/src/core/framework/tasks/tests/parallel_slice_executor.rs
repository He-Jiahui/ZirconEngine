use super::ParallelSliceExecutor;

struct SerialExecutor;

impl ParallelSliceExecutor for SerialExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], _chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        task(items);
    }
}

struct MoveOnly(Box<str>);

#[test]
fn ordered_map_default_moves_items_and_preserves_input_order() {
    let values = vec![MoveOnly("first".into()), MoveOnly("second".into())];

    let output = SerialExecutor.parallel_map_ordered(values, |value| value.0);

    assert_eq!(output, vec![Box::<str>::from("first"), Box::from("second")]);
}
