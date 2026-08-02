fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn schedule_parallel_task_lookup_uses_direct_error_branches() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let task_for_system = section_between(
        source,
        "fn task_for_system<'registry>",
        "fn tasks_for_batch<'registry>",
    );
    let run_task_result = section_between(source, "fn run_task_result<E>", "}");

    assert!(
        task_for_system.contains("let Some(task) = self.tasks.get(system_id) else")
            && task_for_system.contains("return Err(ScheduleParallelExecutorError::MissingTask")
            && task_for_system.contains("Ok(Arc::clone(task))")
            && !task_for_system.contains(".map(|task| task.as_ref())")
            && !task_for_system.contains(".ok_or_else("),
        "parallel task lookup must use a direct missing-task branch"
    );
    assert!(
        run_task_result.contains("match result")
            && run_task_result.contains("Ok(()) => Ok(())")
            && run_task_result
                .contains("Err(error) => Err(ScheduleParallelExecutorError::TaskFailed")
            && !run_task_result.contains(".map_err("),
        "parallel task result handling must use a direct error branch"
    );
}

#[test]
fn schedule_parallel_exact_four_batches_use_fixed_scheduler_join_path() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let exact_four = section_between(
        source,
        "if let [first_system_id, second_system_id, third_system_id, fourth_system_id] =",
        "if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =",
    );

    assert!(
        exact_four.contains("let fourth_task = registry.task_for_system(fourth_system_id)?;")
            && exact_four.contains("scheduler.join(")
            && exact_four.contains(
                "|| left_scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()())"
            )
            && exact_four.contains(
                "|| right_scheduler.join(|| third_task.as_ref()(), || fourth_task.as_ref()())"
            )
            && exact_four.contains("run_task_result(fourth_system_id, fourth_result)?;")
            && exact_four.contains("return Ok(());")
            && !exact_four.contains("Vec::with_capacity")
            && !exact_four.contains("tasks_for_batch"),
        "exact four schedule batches must use the fixed scheduler join path before generic vector preflight"
    );

    let exact_four_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id] =")
        .expect("schedule executor should have an exact-four branch");
    let generic_preflight_index = source
        .find("let tasks = registry.tasks_for_batch(system_ids)?")
        .expect("seven-or-more schedule batches should keep the generic preflight path");
    assert!(exact_four_index < generic_preflight_index);
}

#[test]
fn schedule_parallel_exact_five_batches_use_fixed_scheduler_join_path() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let exact_five = section_between(
        source,
        "if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =",
        "if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =",
    );

    assert!(
        exact_five.contains("let fifth_task = registry.task_for_system(fifth_system_id)?;")
            && exact_five.contains(
                "|| left_scheduler.join(|| first_task.as_ref()(), || second_task.as_ref()())"
            )
            && exact_five.contains("nested_scheduler")
            && exact_five.contains("|| fifth_task.as_ref()()")
            && exact_five.contains("run_task_result(fifth_system_id, fifth_result)?;")
            && exact_five.contains("return Ok(());")
            && !exact_five.contains("Vec::with_capacity")
            && !exact_five.contains("tasks_for_batch"),
        "exact five schedule batches must use the fixed scheduler join path before generic vector preflight"
    );

    let exact_four_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id] =")
        .expect("schedule executor should have an exact-four branch");
    let exact_five_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =")
        .expect("schedule executor should have an exact-five branch");
    let exact_six_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =")
        .expect("schedule executor should have an exact-six branch");
    let generic_preflight_index = source
        .find("let tasks = registry.tasks_for_batch(system_ids)?")
        .expect("seven-or-more schedule batches should keep the generic preflight path");
    assert!(exact_four_index < exact_five_index);
    assert!(exact_five_index < exact_six_index);
    assert!(exact_six_index < generic_preflight_index);
}

#[test]
fn schedule_parallel_exact_six_batches_use_fixed_scheduler_join_path() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let exact_six = section_between(
        source,
        "if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =",
        "let tasks = registry.tasks_for_batch(system_ids)?",
    );

    assert!(
        exact_six.contains("let sixth_task = registry.task_for_system(sixth_system_id)?;")
            && exact_six.contains("first_pair_scheduler")
            && exact_six.contains("second_pair_scheduler")
            && exact_six.contains(
                "|| right_scheduler.join(|| fifth_task.as_ref()(), || sixth_task.as_ref()())"
            )
            && exact_six.contains("run_task_result(sixth_system_id, sixth_result)?;")
            && exact_six.contains("return Ok(());")
            && !exact_six.contains("Vec::with_capacity")
            && !exact_six.contains("tasks_for_batch"),
        "exact six schedule batches must use the fixed scheduler join path before generic vector preflight"
    );

    let exact_five_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id] =")
        .expect("schedule executor should have an exact-five branch");
    let exact_six_index = source
        .find("if let [first_system_id, second_system_id, third_system_id, fourth_system_id, fifth_system_id, sixth_system_id] =")
        .expect("schedule executor should have an exact-six branch");
    let generic_preflight_index = source
        .find("let tasks = registry.tasks_for_batch(system_ids)?")
        .expect("seven-or-more schedule batches should keep the generic preflight path");
    assert!(exact_five_index < exact_six_index);
    assert!(exact_six_index < generic_preflight_index);
}

#[test]
fn schedule_parallel_generic_batch_results_pair_with_borrowed_system_ids() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let generic = section_between(
        source,
        "let tasks = registry.tasks_for_batch(system_ids)?",
        "fn run_serial_batch<E>",
    );

    assert!(
        generic.contains("let results = run_parallel_tasks(scheduler, &tasks);")
            && generic.contains("for (system_id, result) in system_ids.iter().zip(results)")
            && generic.contains("run_task_result(system_id, result)?;")
            && generic.contains("fn run_parallel_tasks<E>")
            && generic.contains("left_results.extend(right_results);")
            && !generic.contains("for (index, result) in results.into_iter().enumerate()")
            && !generic.contains(".get(index)")
            && !generic.contains("task result index must originate from batch order")
            && !generic.contains("ScheduleParallelExecutorError::TaskFailed { system_id, error }"),
        "generic schedule batches must pair borrowed batch ids with scheduler-owned result slots directly instead of looking ids up again by result index"
    );
}

#[test]
fn schedule_parallel_executor_does_not_call_rayon_directly() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");

    assert!(
        !source.contains("rayon::") && !source.contains("use rayon"),
        "schedule parallel execution should reach rayon only through core task primitives"
    );
    assert!(
        source.contains("scheduler.join(")
            && source.contains("run_parallel_tasks(scheduler, &tasks)"),
        "schedule parallel execution should keep fixed and generic parallel paths on JobScheduler"
    );
}

#[test]
fn schedule_parallel_batches_chain_through_job_handles() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let run_with_report = section_between(
        source,
        "pub fn run_batches_with_report<E>(",
        "impl ScheduleParallelExecutionReport",
    );

    assert!(
        run_with_report.contains("let mut previous_batch = JobHandle::completed();")
            && run_with_report.contains("let dependency = previous_batch.clone();")
            && run_with_report.contains("schedule_after(std::slice::from_ref(&dependency)")
            && run_with_report.contains("previous_batch = batch_handle;")
            && run_with_report.contains("previous_batch.wait();")
            && run_with_report.contains("aborted_for_task.store(true, Ordering::Release);"),
        "parallel executor should submit batches through a JobHandle dependency chain and wait only on the tail batch"
    );
}

#[test]
fn schedule_parallel_report_keeps_run_batches_compatible() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let run_batches = section_between(
        source,
        "pub fn run_batches<E>(",
        "pub fn run_batches_with_report<E>(",
    );
    let report = section_between(
        source,
        "pub struct ScheduleParallelExecutionReport {",
        "pub struct ScheduleParallelExecutorError",
    );

    assert!(
        run_batches.contains("self.run_batches_with_report(batches, registry).map(|_| ())")
            && report.contains("parallel_batches: usize")
            && report.contains("serial_batches: usize")
            && report.contains("serial_fallbacks: usize")
            && report.contains("executed_systems: usize"),
        "parallel executor should preserve run_batches while exposing detailed execution reports"
    );
}

#[test]
fn schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts() {
    let source = include_str!("../ecs/schedule_parallel_executor.rs");
    let report_counting = section_between(
        source,
        "if !self.parallel_enabled {",
        "let batch_result = Arc::new(Mutex::new(None));",
    );
    let disabled_execution = section_between(
        source,
        "if !parallel_enabled {",
        "if let [system_id] = system_ids",
    );
    let diagnostics = section_between(
        source,
        "pub fn record_diagnostics(&self, core: &CoreHandle, frame_index: u64)",
        "impl<E> Default for ScheduleParallelTaskRegistry<E>",
    );

    assert!(
        report_counting
            .contains("report.record_serial_batch(system_ids.len(), system_ids.len() > 1)")
            && disabled_execution.contains("return run_serial_batch(system_ids, registry);")
            && diagnostics.contains("SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC")
            && diagnostics.contains("SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC")
            && diagnostics.contains("self.parallel_batches as f64")
            && diagnostics.contains("self.serial_fallbacks as f64"),
        "disabled schedule parallelism should execute serially and publish observable counts"
    );
}
