//! 字节阈值的边界、单次请求分类及报告合并的确定性夹具。
//!
//! 只调用标量构造与记账接口，不进入后端或会话，也不验证调度或成功产物。
//! 合并测试直接提供两份报告，不启动并行工作，且未覆盖计数饱和边界。

use super::{TextShapingWorkBudget, TextShapingWorkReport};

#[test]
fn shaping_work_budget_is_a_non_zero_execution_threshold() {
    assert_eq!(TextShapingWorkBudget::new(0), None);

    let budget = TextShapingWorkBudget::default();
    let boundary = budget.max_inline_input_bytes();

    assert!(!budget.exceeds_inline_threshold(boundary));
    assert!(budget.exceeds_inline_threshold(boundary + 1));
}

#[test]
fn shaping_work_report_classifies_complete_synchronous_requests_without_slicing() {
    let budget = TextShapingWorkBudget::new(8).expect("non-zero budget");
    let mut report = TextShapingWorkReport::default();

    report.record_synchronous_request(budget, 5);
    report.record_synchronous_request(budget, 13);

    assert_eq!(report.inline_request_count, 1);
    assert_eq!(report.oversized_synchronous_request_count, 1);
    assert_eq!(report.synchronous_input_bytes, 18);
    assert_eq!(report.max_synchronous_input_bytes, 13);
}

#[test]
fn shaping_work_report_merges_parallel_batch_receipts() {
    let budget = TextShapingWorkBudget::new(4).expect("non-zero budget");
    let mut first = TextShapingWorkReport::default();
    first.record_synchronous_request(budget, 3);
    let mut second = TextShapingWorkReport::default();
    second.record_synchronous_request(budget, 9);

    first.merge(second);

    assert_eq!(first.inline_request_count, 1);
    assert_eq!(first.oversized_synchronous_request_count, 1);
    assert_eq!(first.synchronous_input_bytes, 12);
    assert_eq!(first.max_synchronous_input_bytes, 9);
}
