use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

use zircon_runtime::core::runtime::tasks::{
    EngineTaskGraph, EngineTaskGraphOptions, TaskCancellationPolicy, TaskDescriptor,
    TaskGraphScopeDescriptor, TaskId, TaskPoolKind,
};

const TOTAL_WORKER_BUDGETS: [usize; 2] = [3, 8];
const SAMPLE_COUNT: usize = 21;

#[test]
#[ignore = "managed Windows release blocking-I/O isolation evidence"]
fn runtime11_task_domain_isolation_profile() {
    for total_worker_budget in TOTAL_WORKER_BUDGETS {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(
            total_worker_budget,
        ))
        .expect("performance profile should create runtime worker domains");
        let inventory = runtime.worker_inventory();
        let io_workers = inventory
            .domain(TaskPoolKind::Io)
            .expect("runtime should report its I/O domain")
            .worker_count;
        let compute_workers = inventory
            .domain(TaskPoolKind::Compute)
            .expect("runtime should report its compute domain")
            .worker_count;
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new(format!(
                "runtime11-domain-isolation-{total_worker_budget}"
            )))
            .expect("performance profile should create one canonical scope");
        let mut compute_latency_ns = Vec::with_capacity(SAMPLE_COUNT);

        for sample in 0..SAMPLE_COUNT {
            let release = Arc::new(Barrier::new(io_workers + 1));
            let (io_started_tx, io_started_rx) = mpsc::sync_channel(io_workers);
            let mut io_tasks = Vec::with_capacity(io_workers);
            for worker in 0..io_workers {
                let release = Arc::clone(&release);
                let io_started_tx = io_started_tx.clone();
                io_tasks.push(
                    scope
                        .submit(
                            descriptor(
                                task_id(sample, worker),
                                TaskPoolKind::Io,
                                "profile-io-blocker",
                            ),
                            move |_| {
                                io_started_tx
                                    .send(())
                                    .expect("profile owner should observe I/O saturation");
                                release.wait();
                            },
                        )
                        .expect("I/O blocker should enter the canonical TaskGraph"),
                );
            }
            drop(io_started_tx);
            for _ in 0..io_workers {
                io_started_rx
                    .recv_timeout(Duration::from_secs(2))
                    .expect("every physical I/O worker should become blocked");
            }

            let (compute_started_tx, compute_started_rx) = mpsc::sync_channel(1);
            let started_at = Instant::now();
            let compute = scope
                .submit(
                    descriptor(
                        task_id(sample, io_workers),
                        TaskPoolKind::Compute,
                        "profile-compute-marker",
                    ),
                    move |_| {
                        compute_started_tx
                            .send(())
                            .expect("profile owner should observe compute progress");
                    },
                )
                .expect("compute marker should enter the canonical TaskGraph");
            compute_started_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("compute must progress before saturated I/O is released");
            compute_latency_ns
                .push(u64::try_from(started_at.elapsed().as_nanos()).unwrap_or(u64::MAX));

            compute.wait();
            release.wait();
            for task in io_tasks {
                task.wait();
            }
        }

        compute_latency_ns.sort_unstable();
        let p50_ns = percentile(&compute_latency_ns, 50);
        let p95_ns = percentile(&compute_latency_ns, 95);
        println!(
            "RUNTIME11_TASK_DOMAIN_ISOLATION_PROFILE_V1 total_worker_budget={total_worker_budget} io_workers={io_workers} compute_workers={compute_workers} samples={SAMPLE_COUNT} compute_before_io_release={SAMPLE_COUNT} compute_latency_p50_ns={p50_ns} compute_latency_p95_ns={p95_ns}"
        );

        runtime
            .shutdown(Duration::from_secs(2))
            .expect("profile runtime should join every worker domain");
    }
}

fn descriptor(id: TaskId, kind: TaskPoolKind, label: &str) -> TaskDescriptor {
    TaskDescriptor::new(id, kind, label)
        .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown)
}

fn task_id(sample: usize, lane_index: usize) -> TaskId {
    TaskId::new(
        1 + u64::try_from(sample)
            .expect("sample index should fit in task identity")
            .saturating_mul(1_000)
            .saturating_add(
                u64::try_from(lane_index).expect("lane index should fit in task identity"),
            ),
    )
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let rank = sorted
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    sorted[rank]
}
