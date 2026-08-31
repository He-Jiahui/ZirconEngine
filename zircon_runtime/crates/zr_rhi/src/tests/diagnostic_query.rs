use crate::{
    aggregate_diagnostic_query_results, DiagnosticPipelineStatistics, DiagnosticQueryPlan,
    DiagnosticQueryPlanError, DiagnosticReadbackBudget,
};

fn budget() -> DiagnosticReadbackBudget {
    DiagnosticReadbackBudget::default().with_query_limits(2, 2, 3)
}

fn bytes(values: &[u64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

#[test]
fn query_plan_admits_zero_single_and_exact_budget_scopes_before_rejecting_more() {
    let mut plan = DiagnosticQueryPlan::new(budget());
    assert!(aggregate_diagnostic_query_results(&plan, &[], &[])
        .unwrap()
        .is_empty());

    let first = plan.register_pass().unwrap();
    let second = plan.register_pass().unwrap();
    assert!(matches!(
        plan.register_pass(),
        Err(DiagnosticQueryPlanError::PassLimitExceeded { .. })
    ));
    assert!(plan.reserve_timestamp_scope(first).is_ok());
    assert!(plan.reserve_timestamp_scope(second).is_ok());
    assert!(matches!(
        plan.reserve_timestamp_scope(first),
        Err(DiagnosticQueryPlanError::TimestampScopeLimitExceeded { .. })
    ));
    assert!(plan.reserve_pipeline_statistics_scope(first).is_ok());
    assert!(plan.reserve_pipeline_statistics_scope(first).is_ok());
    assert!(plan.reserve_pipeline_statistics_scope(second).is_ok());
    assert!(matches!(
        plan.reserve_pipeline_statistics_scope(second),
        Err(DiagnosticQueryPlanError::PipelineStatisticsScopeLimitExceeded { .. })
    ));
}

#[test]
fn query_plan_rejects_cross_pass_or_unowned_pass_scope_bindings() {
    let mut plan = DiagnosticQueryPlan::for_frame(77, budget());
    let first = plan.register_pass().unwrap();
    let second = plan.register_pass().unwrap();
    let first_timestamp = plan.reserve_timestamp_scope(first).unwrap();
    let second_statistics = plan.reserve_pipeline_statistics_scope(second).unwrap();

    assert!(matches!(
        plan.pass_scope(Some(first_timestamp), Some(second_statistics)),
        Err(DiagnosticQueryPlanError::ScopePassMismatch)
    ));
    assert_eq!(plan.frame_index(), Some(77));
}

#[test]
fn query_decode_rejects_truncated_or_wrong_query_byte_counts() {
    let mut plan = DiagnosticQueryPlan::new(budget());
    let pass = plan.register_pass().unwrap();
    plan.reserve_timestamp_scope(pass).unwrap();
    plan.reserve_pipeline_statistics_scope(pass).unwrap();

    assert!(aggregate_diagnostic_query_results(&plan, &[0; 8], &bytes(&[1, 2, 3, 4, 5])).is_err());
    assert!(aggregate_diagnostic_query_results(&plan, &bytes(&[1, 2]), &[0; 32]).is_err());
}

#[test]
fn pipeline_statistics_scopes_use_one_query_index_and_five_result_values_each() {
    let mut plan = DiagnosticQueryPlan::new(budget());
    let pass = plan.register_pass().unwrap();
    let first = plan.reserve_pipeline_statistics_scope(pass).unwrap();
    let second = plan.reserve_pipeline_statistics_scope(pass).unwrap();

    assert_eq!(first.query_index(), 0);
    assert_eq!(second.query_index(), 1);
    assert_eq!(plan.pipeline_statistics_query_count(), 2);
    assert_eq!(plan.pipeline_statistics_result_value_count(), 10);

    let result = aggregate_diagnostic_query_results(
        &plan,
        &[],
        &bytes(&[1, 2, 3, 4, 5, 10, 20, 30, 40, 50]),
    )
    .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].pipeline_statistics,
        DiagnosticPipelineStatistics {
            vertex_shader_invocations: 11,
            clipper_invocations: 22,
            clipper_primitives_out: 33,
            fragment_shader_invocations: 44,
            compute_shader_invocations: 55,
        }
    );
}

#[test]
fn query_decode_aggregates_repeated_physical_scopes_by_dense_pass_id_in_linear_order() {
    let mut plan = DiagnosticQueryPlan::new(budget());
    let hzb = plan.register_pass().unwrap();
    let ui = plan.register_pass().unwrap();
    plan.reserve_timestamp_scope(hzb).unwrap();
    plan.reserve_timestamp_scope(hzb).unwrap();
    plan.reserve_pipeline_statistics_scope(hzb).unwrap();
    plan.reserve_pipeline_statistics_scope(hzb).unwrap();
    plan.reserve_pipeline_statistics_scope(ui).unwrap();

    let result = aggregate_diagnostic_query_results(
        &plan,
        &bytes(&[10, 18, 30, 45]),
        &bytes(&[1, 2, 3, 4, 5, 10, 20, 30, 40, 50, 7, 8, 9, 10, 11]),
    )
    .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].pass, hzb);
    assert_eq!(result[0].timestamp_ticks, 23);
    assert_eq!(
        result[0].pipeline_statistics,
        DiagnosticPipelineStatistics {
            vertex_shader_invocations: 11,
            clipper_invocations: 22,
            clipper_primitives_out: 33,
            fragment_shader_invocations: 44,
            compute_shader_invocations: 55,
        }
    );
    assert_eq!(result[1].pass, ui);
    assert_eq!(result[1].timestamp_ticks, 0);
    assert_eq!(result[1].pipeline_statistics.compute_shader_invocations, 11);
}
