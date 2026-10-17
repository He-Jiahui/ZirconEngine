//! 用源码断言约束候选容量预留语句的保留。
//! 读取实现的测试挂载前部分，不执行派发，也不测量实际分配或性能。

#[test]
fn navigation_dispatch_reserves_candidate_invocation_upper_bound() {
    let source = include_str!("../dispatcher.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("navigation dispatcher implementation before tests");

    assert!(implementation.contains("let mut invocations = Vec::with_capacity(candidates.len());"));
}
