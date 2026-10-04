//! Real concurrent admission and completion evidence, with bounded in-flight work.
//! Timing is machine evidence only; no baseline or numeric acceptance budget exists.

use std::collections::HashSet;
use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EngineEventPublishRejection, EngineEventSubscription,
    EventBudgetScope, EventBusDiagnosticsMode, EventBusLimits, EventRetentionLimits,
};
use crate::core::{EngineEvent, EventBus};

const PRODUCERS: usize = 2;
const WORK_PER_PRODUCER: usize = 32;
const SAFETY_TIMEOUT: Duration = Duration::from_secs(120);
const FANOUTS: [usize; 4] = [1, 2, 5, 100];
const PAYLOADS: [usize; 3] = [64, 4_096, 262_144];

fn ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn owned_input(topic: &str, producer: usize, sequence: usize, bytes: usize) -> EngineEvent {
    EngineEvent {
        topic: topic.into(),
        payload: serde_json::json!({"producer": producer, "sequence": sequence, "blob": "x".repeat(bytes)}),
    }
}

// Checksum is a compact observation of the actual owned rejection, not an
// allocation metric or a cryptographic claim. Full equality is asserted too.
fn owner_observation(event: &EngineEvent) -> serde_json::Value {
    let blob = event.payload["blob"].as_str().expect("owned payload blob");
    let checksum = blob
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325_u64, |sum, byte| {
            (sum ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    serde_json::json!({"topic": event.topic, "producer": event.payload["producer"],
        "sequence": event.payload["sequence"], "blob_bytes": blob.len(), "blob_fnv1a64": checksum})
}

struct ConsumerOutput {
    receive_ns: Vec<u64>,
    release_batch_ns: Vec<u64>,
    identities: HashSet<(usize, usize)>,
    deliveries: usize,
    released_bytes: usize,
}

struct ProducerOutput {
    attempts: Vec<serde_json::Value>,
    complete_ns: Vec<u64>,
    identities: HashSet<(usize, usize)>,
    admitted_bytes: usize,
}

enum WorkerOutput {
    Producer(ProducerOutput),
    Consumer(ConsumerOutput),
    Failed(String),
}

fn worker_result(work: impl FnOnce() -> WorkerOutput) -> WorkerOutput {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|failure| {
        let message = failure
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                failure
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
            })
            .unwrap_or_else(|| "worker panic without string payload".to_string());
        WorkerOutput::Failed(message)
    })
}

fn consume(
    subscriptions: Vec<Box<dyn EngineEventSubscription>>,
    expected_publications: usize,
    payload_bytes: usize,
    expected_topic: String,
    allowed_producers: Vec<usize>,
    acknowledgements: Vec<mpsc::Sender<usize>>,
) -> ConsumerOutput {
    let mut output = ConsumerOutput {
        receive_ns: Vec::with_capacity(expected_publications * subscriptions.len()),
        release_batch_ns: Vec::with_capacity(expected_publications),
        identities: HashSet::with_capacity(expected_publications),
        deliveries: 0,
        released_bytes: 0,
    };
    for _ in 0..expected_publications {
        let batch_started = Instant::now();
        let mut identity = None;
        for subscription in &subscriptions {
            let started = Instant::now();
            let event = subscription
                .recv_timeout(SAFETY_TIMEOUT)
                .expect("real consumer delivery");
            output.receive_ns.push(ns(started.elapsed()));
            assert_eq!(
                event.topic(),
                expected_topic.as_str(),
                "delivery must stay on its subscribed topic"
            );
            let payload = event
                .decode_payload()
                .expect("decode current frozen contract");
            let key = (
                payload["producer"].as_u64().unwrap() as usize,
                payload["sequence"].as_u64().unwrap() as usize,
            );
            assert!(
                allowed_producers.contains(&key.0),
                "this topic received a publication from the wrong producer"
            );
            assert!(
                key.1 < WORK_PER_PRODUCER,
                "delivery sequence is outside the producer workload"
            );
            let expected = owned_input(&expected_topic, key.0, key.1, payload_bytes);
            assert_eq!(
                payload, expected.payload,
                "every decoded field and blob byte must equal the original producer input"
            );
            assert_eq!(payload["blob"].as_str().unwrap().len(), payload_bytes);
            if let Some(expected) = identity {
                assert_eq!(key, expected, "fanout order must agree");
            } else {
                identity = Some(key);
            }
            output.deliveries += 1;
            output.released_bytes += event.retained_bytes();
            drop(event);
        }
        let (producer, sequence) = identity.expect("nonempty fanout");
        assert!(
            output.identities.insert((producer, sequence)),
            "no duplicate delivery"
        );
        output.release_batch_ns.push(ns(batch_started.elapsed()));
        // Acknowledge only after every fanout handle has actually released.
        acknowledgements[producer]
            .send(sequence)
            .expect("producer completion owner");
    }
    output
}

fn produce(
    bus: EventBus,
    producer: usize,
    topic: String,
    fanout: usize,
    payload_bytes: usize,
    acknowledgement: mpsc::Receiver<usize>,
) -> ProducerOutput {
    let mut output = ProducerOutput {
        attempts: Vec::new(),
        complete_ns: Vec::with_capacity(WORK_PER_PRODUCER),
        identities: HashSet::with_capacity(WORK_PER_PRODUCER),
        admitted_bytes: 0,
    };
    for sequence in 0..WORK_PER_PRODUCER {
        let expected = owned_input(&topic, producer, sequence, payload_bytes);
        let mut input = expected.clone();
        let complete_started = Instant::now();
        let deadline = complete_started + SAFETY_TIMEOUT;
        loop {
            assert!(
                Instant::now() < deadline,
                "bounded producer retry timed out"
            );
            let started = Instant::now();
            let result = bus.try_publish(input);
            let publish_ns = ns(started.elapsed());
            match result {
                Ok(receipt) => {
                    assert_eq!(receipt.subscribers, fanout);
                    assert_eq!(receipt.replaced_events, 0);
                    assert_eq!(receipt.replaced_bytes, 0);
                    output.admitted_bytes += receipt.admitted_bytes;
                    output.attempts.push(serde_json::json!({"producer": producer, "sequence": sequence,
                        "publish_ns": publish_ns, "result": "admitted", "recipients": receipt.subscribers,
                        "admitted_bytes": receipt.admitted_bytes}));
                    break;
                }
                Err(rejected) => {
                    assert_eq!(
                        rejected.event, expected,
                        "rejection preserves original owned event"
                    );
                    assert!(
                        matches!(
                            rejected.reason,
                            EngineEventPublishRejection::Backpressured { .. }
                        ),
                        "unexpected real admission failure: {:?}",
                        rejected.reason
                    );
                    output.attempts.push(serde_json::json!({"producer": producer, "sequence": sequence,
                        "publish_ns": publish_ns, "result": "rejected", "reason": format!("{:?}", rejected.reason),
                        "owner": owner_observation(&rejected.event), "owner_equals_original": true}));
                    input = rejected.event;
                    thread::yield_now();
                }
            }
        }
        assert_eq!(
            acknowledgement
                .recv_timeout(SAFETY_TIMEOUT)
                .expect("actual consumer completion"),
            sequence
        );
        output.complete_ns.push(ns(complete_started.elapsed()));
        assert!(output.identities.insert((producer, sequence)));
    }
    output
}

fn contention(
    mode: EventBusDiagnosticsMode,
    separate_topics: bool,
    fanout: usize,
    payload_bytes: usize,
) {
    let limits = EventBusLimits::default();
    let bus = EventBus::new(mode);
    let topics = if separate_topics {
        vec!["runtime.contention.a", "runtime.contention.b"]
    } else {
        vec!["runtime.contention.same"]
    };
    let exact_bytes: Vec<_> = topics
        .iter()
        .map(|topic| {
            let input = owned_input(topic, PRODUCERS - 1, WORK_PER_PRODUCER - 1, payload_bytes);
            topic.len() + serde_json::to_vec(&input.payload).unwrap().len()
        })
        .collect();
    let capacity_per_topic: Vec<_> = exact_bytes
        .iter()
        .map(|bytes| {
            (limits.topic.max_events.get() / fanout)
                .min(limits.topic.max_bytes.get() / (bytes * fanout))
        })
        .collect();
    assert!(
        capacity_per_topic.iter().all(|capacity| *capacity >= 1),
        "every declared fanout fits actual defaults"
    );
    assert!(exact_bytes
        .iter()
        .all(|bytes| bytes * fanout <= limits.global.max_bytes.get()));
    // At most one unacknowledged event per producer. Default capacities may
    // cause real typed pressure for the largest same-topic fanout; retry owns
    // that same event and consumer progress releases the capacity.
    let (ready_tx, ready_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel::<WorkerOutput>();
    let mut starts = Vec::new();
    let mut workers = Vec::new();
    let mut ack_senders = Vec::new();
    let mut ack_receivers = Vec::new();
    for _ in 0..PRODUCERS {
        let (sender, receiver) = mpsc::channel();
        ack_senders.push(sender);
        ack_receivers.push(receiver);
    }
    for (topic_index, topic) in topics.iter().enumerate() {
        let expected_topic = (*topic).to_string();
        let allowed_producers = if separate_topics {
            vec![topic_index]
        } else {
            (0..PRODUCERS).collect()
        };
        let subscriptions = (0..fanout)
            .map(|_| {
                bus.subscribe(
                    *topic,
                    EngineEventDeliveryPolicy::Reliable {
                        limits: EventRetentionLimits::default(),
                    },
                )
                .unwrap()
            })
            .collect();
        let publications = if separate_topics {
            WORK_PER_PRODUCER
        } else {
            WORK_PER_PRODUCER * PRODUCERS
        };
        let acknowledgements = ack_senders.clone();
        let (start_tx, start_rx) = mpsc::channel();
        starts.push(start_tx);
        let ready = ready_tx.clone();
        let done = done_tx.clone();
        workers.push(thread::spawn(move || {
            ready.send(()).unwrap();
            let result = worker_result(|| {
                start_rx
                    .recv_timeout(SAFETY_TIMEOUT)
                    .expect("consumer start signal");
                WorkerOutput::Consumer(consume(
                    subscriptions,
                    publications,
                    payload_bytes,
                    expected_topic,
                    allowed_producers,
                    acknowledgements,
                ))
            });
            done.send(result).unwrap();
        }));
    }
    for (producer, acknowledgement) in ack_receivers.into_iter().enumerate() {
        let topic = topics[if separate_topics { producer } else { 0 }].to_string();
        let producer_bus = bus.clone();
        let (start_tx, start_rx) = mpsc::channel();
        starts.push(start_tx);
        let ready = ready_tx.clone();
        let done = done_tx.clone();
        workers.push(thread::spawn(move || {
            ready.send(()).unwrap();
            let result = worker_result(|| {
                start_rx
                    .recv_timeout(SAFETY_TIMEOUT)
                    .expect("producer start signal");
                WorkerOutput::Producer(produce(
                    producer_bus,
                    producer,
                    topic,
                    fanout,
                    payload_bytes,
                    acknowledgement,
                ))
            });
            done.send(result).unwrap();
        }));
    }
    drop(ready_tx);
    drop(done_tx);
    drop(ack_senders);
    for _ in &workers {
        ready_rx
            .recv_timeout(SAFETY_TIMEOUT)
            .expect("every real worker ready");
    }
    let rss_before = super::current_process_rss_bytes();
    let wall_started = Instant::now();
    for start in starts {
        start.send(()).expect("real worker must start");
    }
    let mut attempts = Vec::new();
    let mut complete_ns = Vec::new();
    let mut receive_ns = Vec::new();
    let mut release_batch_ns = Vec::new();
    let mut admitted = HashSet::new();
    let mut delivered = HashSet::new();
    let mut admitted_bytes = 0;
    let mut released_bytes = 0;
    let mut deliveries = 0;
    let mut failures = Vec::new();
    for _ in &workers {
        match done_rx
            .recv_timeout(SAFETY_TIMEOUT)
            .expect("every producer/consumer must actually finish")
        {
            WorkerOutput::Producer(output) => {
                attempts.extend(output.attempts);
                complete_ns.extend(output.complete_ns);
                for key in output.identities {
                    assert!(admitted.insert(key));
                }
                admitted_bytes += output.admitted_bytes;
            }
            WorkerOutput::Consumer(output) => {
                receive_ns.extend(output.receive_ns);
                release_batch_ns.extend(output.release_batch_ns);
                for key in output.identities {
                    assert!(delivered.insert(key));
                }
                deliveries += output.deliveries;
                released_bytes += output.released_bytes;
            }
            WorkerOutput::Failed(message) => {
                failures.push(message);
                // A real worker failure closes queues to wake peers. Their
                // terminal results still have to arrive; do not call a failed
                // worker a successful fixed-count completion.
                bus.close_admission();
            }
        }
    }
    let wall_elapsed = wall_started.elapsed();
    // Completion messages are sent after the work. Join also checks panic;
    // timeout above fails before any unbounded join on an unfinished worker.
    for worker in workers {
        worker.join().expect("real worker failure");
    }
    assert!(
        failures.is_empty(),
        "actual producer/consumer failures: {:?}",
        failures
    );
    let rss_after = super::current_process_rss_bytes();
    assert_eq!(admitted, delivered);
    assert_eq!(admitted.len(), PRODUCERS * WORK_PER_PRODUCER);
    assert_eq!(deliveries, admitted.len() * fanout);
    assert_eq!(admitted_bytes, released_bytes);
    let retention = bus.retention_snapshot();
    assert_eq!(retention.retained_events, 0);
    assert_eq!(retention.retained_bytes, 0);
    assert!(retention.peak_retained_events <= limits.global.max_events.get());
    assert!(retention.peak_retained_bytes <= limits.global.max_bytes.get());
    assert!(!retention.poisoned);
    assert!(!retention.closed);
    let diagnostics = bus.diagnostic_report();
    if mode != EventBusDiagnosticsMode::Disabled {
        assert_eq!(diagnostics.published as usize, attempts.len());
        assert_eq!(diagnostics.delivered as usize, deliveries);
        assert_eq!(diagnostics.dropped, 0);
        assert_eq!(diagnostics.queued, 0);
    } else {
        assert_eq!(diagnostics.published, 0);
        assert_eq!(diagnostics.delivered, 0);
    }
    let publish_ns: Vec<_> = attempts
        .iter()
        .map(|row| row["publish_ns"].as_u64().unwrap())
        .collect();
    let rejected_attempts = attempts
        .iter()
        .filter(|row| row["result"] == "rejected")
        .count();
    println!(
        "EVENTBUS_CONTENTION_V1 {}",
        serde_json::json!({
            "topology": if separate_topics { "different_topics" } else { "same_topic" },
            "mode": super::event_bus_diagnostics_mode_label(mode), "producers": PRODUCERS,
            "fanout_per_publication": fanout, "consumer_workers": topics.len(), "payload_bytes": payload_bytes,
            "work_per_producer": WORK_PER_PRODUCER, "max_unacknowledged_per_producer": 1,
            "default_topic_capacity_publications": capacity_per_topic, "frozen_bytes_per_delivery": exact_bytes,
            "attempts": attempts, "publish_raw_ns": publish_ns,
            "receive_raw_ns": receive_ns, "complete_raw_ns": complete_ns,
            "release_batch_raw_ns": release_batch_ns, "publish_percentiles": percentiles(&publish_ns),
            "receive_percentiles": percentiles(&receive_ns), "complete_percentiles": percentiles(&complete_ns),
            "wall_ns": ns(wall_elapsed), "wall_publications_per_second": admitted.len() as f64 / wall_elapsed.as_secs_f64(),
            "wall_deliveries_per_second": deliveries as f64 / wall_elapsed.as_secs_f64(),
            "attempt_count": attempts.len(), "rejected_attempts": rejected_attempts,
            "admitted": admitted.len(), "delivered": deliveries, "admitted_bytes": admitted_bytes,
            "released_bytes": released_bytes, "final_retained_events": retention.retained_events,
            "final_retained_bytes": retention.retained_bytes, "peak_retained_bytes": retention.peak_retained_bytes,
            "peak_retained_events": retention.peak_retained_events,
            "rss_before": rss_before, "rss_after": rss_after, "allocation_measurement": "pending_external_managed_profile",
            "baseline_and_regression_budget": "pending", "latency_bound": "none; safety timeout detects stalled work"
        })
    );
}

fn percentiles(samples: &[u64]) -> serde_json::Value {
    assert!(!samples.is_empty(), "actual samples required");
    serde_json::json!({"p50_ns": super::percentile_ns(samples, 50),
        "p95_ns": super::percentile_ns(samples, 95), "p99_ns": super::percentile_ns(samples, 99)})
}

fn held_clone_pressure(mode: EventBusDiagnosticsMode) {
    let bus = EventBus::new(mode);
    let topic = "runtime.contention.held";
    let sub = bus
        .subscribe(
            topic,
            EngineEventDeliveryPolicy::Reliable {
                limits: EventRetentionLimits {
                    max_events: NonZeroUsize::new(1).unwrap(),
                    max_bytes: NonZeroUsize::new(4_096).unwrap(),
                },
            },
        )
        .unwrap();
    let first = owned_input(topic, 0, 0, 64);
    let next = owned_input(topic, 0, 1, 64);
    let expected_first = first.clone();
    let expected_retry = next.clone();
    let started = Instant::now();
    let admitted = bus.try_publish(first).unwrap();
    let publish_ns = ns(started.elapsed());
    let (commands, command_rx) = mpsc::channel();
    let (stages, stage_rx) = mpsc::channel();
    let consumer = thread::spawn(move || {
        let started = Instant::now();
        let received = sub.recv_timeout(SAFETY_TIMEOUT).unwrap();
        let receive_ns = ns(started.elapsed());
        assert_eq!(received.topic(), expected_first.topic.as_str());
        let received_payload = received.decode_payload().unwrap();
        assert_eq!(
            received_payload, expected_first.payload,
            "held first delivery equals original event"
        );
        assert_eq!(received_payload["sequence"], 0);
        let clone = received.clone();
        stages.send(("held", receive_ns)).unwrap();
        command_rx
            .recv_timeout(SAFETY_TIMEOUT)
            .expect("release first handle command");
        drop(received);
        stages.send(("clone_only", 0)).unwrap();
        command_rx
            .recv_timeout(SAFETY_TIMEOUT)
            .expect("release last clone command");
        drop(clone);
        stages.send(("released", 0)).unwrap();
        let started = Instant::now();
        let retry = sub.recv_timeout(SAFETY_TIMEOUT).unwrap();
        let receive_ns = ns(started.elapsed());
        assert_eq!(retry.topic(), expected_retry.topic.as_str());
        let retry_payload = retry.decode_payload().unwrap();
        assert_eq!(
            retry_payload, expected_retry.payload,
            "held retry delivery equals original event"
        );
        assert_eq!(retry_payload["sequence"], 1);
        drop(retry);
        stages.send(("retry_delivered", receive_ns)).unwrap();
    });
    let (stage, receive_ns) = stage_rx
        .recv_timeout(SAFETY_TIMEOUT)
        .expect("consumer held clone");
    assert_eq!(stage, "held");
    let held = bus.retention_snapshot();
    assert_eq!(held.retained_events, 1);
    assert_eq!(held.retained_bytes, admitted.admitted_bytes);
    let started = Instant::now();
    let rejected = bus.try_publish(next.clone()).unwrap_err();
    let rejection_ns = ns(started.elapsed());
    assert_eq!(
        rejected.reason,
        EngineEventPublishRejection::Backpressured {
            scope: EventBudgetScope::Subscriber
        }
    );
    assert_eq!(rejected.event, next);
    let owner = owner_observation(&rejected.event);
    let reason = format!("{:?}", rejected.reason);
    commands.send(()).unwrap();
    assert_eq!(
        stage_rx.recv_timeout(SAFETY_TIMEOUT).unwrap().0,
        "clone_only"
    );
    assert_eq!(bus.retention_snapshot().retained_bytes, held.retained_bytes);
    commands.send(()).unwrap();
    assert_eq!(stage_rx.recv_timeout(SAFETY_TIMEOUT).unwrap().0, "released");
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
    let started = Instant::now();
    let retry = bus.try_publish(rejected.event).unwrap();
    let retry_ns = ns(started.elapsed());
    let (stage, retry_receive_ns) = stage_rx
        .recv_timeout(SAFETY_TIMEOUT)
        .expect("actual retry delivery");
    assert_eq!(stage, "retry_delivered");
    consumer.join().unwrap();
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
    assert_eq!(retry.subscribers, 1);
    println!(
        "EVENTBUS_CONTENTION_V1 {}",
        serde_json::json!({"kind": "held_clone_typed_pressure",
        "mode": super::event_bus_diagnostics_mode_label(mode), "publish_raw_ns": [publish_ns, rejection_ns, retry_ns],
        "receive_raw_ns": [receive_ns, retry_receive_ns], "rejection": reason,
        "owner": owner, "owner_equals_original": true, "held_bytes": held.retained_bytes,
        "admitted_recipients": admitted.subscribers + retry.subscribers,
        "admitted_bytes": admitted.admitted_bytes + retry.admitted_bytes,
        "final_retained_bytes": bus.retention_snapshot().retained_bytes})
    );
}

#[test]
#[ignore = "managed Release current-contract same/different-topic contention evidence"]
fn runtime02_admission_contention_release_profile() {
    // Alternate paired on/off order for exactly the same workload; sampled
    // diagnostics uses the existing interval. All dimensions run in one ticket.
    let mut case = 0;
    for payload_bytes in PAYLOADS {
        for fanout in FANOUTS {
            for separate_topics in [false, true] {
                let mut modes = [
                    EventBusDiagnosticsMode::Enabled,
                    EventBusDiagnosticsMode::Disabled,
                ];
                if case % 2 != 0 {
                    modes.reverse();
                }
                for mode in modes {
                    contention(mode, separate_topics, fanout, payload_bytes);
                }
                contention(
                    EventBusDiagnosticsMode::Sampled {
                        every: NonZeroU64::new(64).unwrap(),
                    },
                    separate_topics,
                    fanout,
                    payload_bytes,
                );
                case += 1;
            }
        }
    }
    for mode in [
        EventBusDiagnosticsMode::Enabled,
        EventBusDiagnosticsMode::Disabled,
    ] {
        held_clone_pressure(mode);
    }
}
