import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";
import test from "node:test";

import { collectDeliveryActions } from "../src/projections/deliveryActions.ts";
import { selectSourceEngineChoices } from "../src/projections/sourceEngineChoices.ts";

const SAMPLE_PAIRS = 21;
const ITERATIONS_PER_SAMPLE = 20;
const WORKLOAD_SIZE = 10_000;

test("delivery projection partitions package and install history in stable order", () => {
  const actions = [
    { id: "package-1", kind: "package-project" },
    { id: "other-1", kind: "open-output" },
    { id: "install-1", kind: "install-project" },
    { id: "package-2", kind: "package-project" },
    { id: "install-2", kind: "install-project" },
  ];

  const projection = collectDeliveryActions(actions);

  assert.deepEqual(projection.packageActions.map((action) => action.id), ["package-1", "package-2"]);
  assert.deepEqual(projection.installActions.map((action) => action.id), ["install-1", "install-2"]);
});

test("source engine projection preserves selection precedence and caps fallbacks", () => {
  const engines = [
    { id: "engine-a", active: false },
    { id: "engine-b", active: false },
    { id: "engine-c", active: true },
    { id: "engine-d", active: false },
  ];

  const explicit = selectSourceEngineChoices(engines, "engine-b");
  assert.deepEqual(explicit.activeEngines.map((engine) => engine.id), ["engine-b"]);
  assert.deepEqual(explicit.fallbackEngines.map((engine) => engine.id), ["engine-a", "engine-c"]);

  const configured = selectSourceEngineChoices(engines);
  assert.deepEqual(configured.activeEngines.map((engine) => engine.id), ["engine-c"]);
  assert.deepEqual(configured.fallbackEngines.map((engine) => engine.id), ["engine-a", "engine-b"]);

  const missing = selectSourceEngineChoices(engines, "missing-engine");
  assert.deepEqual(missing.activeEngines, []);
  assert.deepEqual(missing.fallbackEngines.map((engine) => engine.id), ["engine-a", "engine-b"]);

  const noConfiguredActive = selectSourceEngineChoices(
    engines.map((engine) => ({ ...engine, active: false })),
  );
  assert.deepEqual(noConfiguredActive.activeEngines.map((engine) => engine.id), ["engine-a"]);
  assert.deepEqual(noConfiguredActive.fallbackEngines.map((engine) => engine.id), ["engine-b", "engine-c"]);
});

test("Cloud and Source Engine pages consume the bounded projections", async () => {
  const [cloudPage, sourceEnginePopover] = await Promise.all([
    readFile(new URL("../src/pages/CloudPage.tsx", import.meta.url), "utf8"),
    readFile(new URL("../src/components/overlays/SourceEnginePopover.tsx", import.meta.url), "utf8"),
  ]);

  assert.match(cloudPage, /collectDeliveryActions\(state\.actionHistory\)/);
  assert.doesNotMatch(cloudPage, /state\.actionHistory\.filter/);
  assert.match(sourceEnginePopover, /selectSourceEngineChoices\(engines, activeEngineId\)/);
  assert.match(sourceEnginePopover, /useMemo/);
  assert.doesNotMatch(sourceEnginePopover, /engines\.filter/);
});

test(
  "page projections meet the Hub04 10k P95 gates",
  { skip: process.env.ZIRCON_HUB04_PERF !== "1" },
  () => {
    const actions = Array.from({ length: WORKLOAD_SIZE }, (_, index) => ({
      id: `action-${index}`,
      kind:
        index % 3 === 0
          ? "package-project"
          : index % 3 === 1
            ? "install-project"
            : "open-output",
    }));
    const engines = Array.from({ length: WORKLOAD_SIZE }, (_, index) => ({
      id: `engine-${index}`,
      active: index === WORKLOAD_SIZE - 1,
    }));

    for (let warmup = 0; warmup < 20; warmup += 1) {
      legacyDeliveryActions(actions);
      collectDeliveryActions(actions);
      legacySourceEngineChoices(engines);
      selectSourceEngineChoices(engines);
    }

    const delivery = pairedSamples(
      () => repeatProjection(() => legacyDeliveryActions(actions)),
      () => repeatProjection(() => collectDeliveryActions(actions)),
    );
    const sourceEngines = pairedSamples(
      () => repeatProjection(() => legacySourceEngineChoices(engines)),
      () => repeatProjection(() => selectSourceEngineChoices(engines)),
    );

    const deliveryLegacyP95 = nearestRank(delivery.legacy, 95);
    const deliveryOptimizedP95 = nearestRank(delivery.optimized, 95);
    const sourceLegacyP95 = nearestRank(sourceEngines.legacy, 95);
    const sourceOptimizedP95 = nearestRank(sourceEngines.optimized, 95);
    assert.equal(delivery.legacyChecksum, delivery.optimizedChecksum);
    assert.equal(sourceEngines.legacyChecksum, sourceEngines.optimizedChecksum);
    assert.ok(
      deliveryOptimizedP95 * 100 <= deliveryLegacyP95 * 50,
      `delivery projection P95 ${deliveryOptimizedP95}ns must be at most 50% of legacy ${deliveryLegacyP95}ns`,
    );
    assert.ok(
      sourceOptimizedP95 * 100 <= sourceLegacyP95 * 80,
      `source-engine projection P95 ${sourceOptimizedP95}ns must be at most 80% of legacy ${sourceLegacyP95}ns`,
    );

    console.log(
      performanceRow(
        "HUB04_DELIVERY_ACTION_PROJECTION_10K_BENCH_V1",
        delivery,
        "legacy_item_checks=20000 optimized_item_checks=10000 threshold=optimized_p95_lte_50pct_legacy",
      ),
    );
    console.log(
      performanceRow(
        "HUB04_SOURCE_ENGINE_CHOICES_10K_BENCH_V1",
        sourceEngines,
        "legacy_fallback_refs=9999 optimized_fallback_refs=2 threshold=optimized_p95_lte_80pct_legacy",
      ),
    );
  },
);

function legacyDeliveryActions(actions) {
  return {
    packageActions: actions.filter((action) => action.kind === "package-project"),
    installActions: actions.filter((action) => action.kind === "install-project"),
  };
}

function legacySourceEngineChoices(engines, activeEngineId) {
  const activeId = activeEngineId ?? engines.find((engine) => engine.active)?.id ?? engines[0]?.id;
  return {
    activeEngines: engines.filter((engine) => engine.id === activeId),
    fallbackEngines: engines.filter((engine) => engine.id !== activeId).slice(0, 2),
  };
}

function repeatProjection(project) {
  let checksum = 0;
  for (let iteration = 0; iteration < ITERATIONS_PER_SAMPLE; iteration += 1) {
    const result = project();
    const groups = Object.values(result);
    checksum += groups[0].length + groups[1].length;
  }
  return checksum;
}

function pairedSamples(legacy, optimized) {
  const legacySamples = [];
  const optimizedSamples = [];
  let legacyChecksum = 0;
  let optimizedChecksum = 0;
  for (let sample = 0; sample < SAMPLE_PAIRS; sample += 1) {
    globalThis.gc?.();
    const first = sample % 2 === 0 ? elapsedNanoseconds(legacy) : elapsedNanoseconds(optimized);
    const second = sample % 2 === 0 ? elapsedNanoseconds(optimized) : elapsedNanoseconds(legacy);
    legacySamples.push(sample % 2 === 0 ? first.elapsed : second.elapsed);
    optimizedSamples.push(sample % 2 === 0 ? second.elapsed : first.elapsed);
    legacyChecksum += sample % 2 === 0 ? first.result : second.result;
    optimizedChecksum += sample % 2 === 0 ? second.result : first.result;
  }
  return {
    legacy: legacySamples,
    optimized: optimizedSamples,
    legacyChecksum,
    optimizedChecksum,
  };
}

function elapsedNanoseconds(operation) {
  const startedAt = performance.now();
  const result = operation();
  return {
    elapsed: Math.max(1, Math.round((performance.now() - startedAt) * 1_000_000)),
    result,
  };
}

function nearestRank(samples, percentile) {
  const sorted = [...samples].sort((left, right) => left - right);
  return sorted[Math.ceil((sorted.length * percentile) / 100) - 1];
}

function performanceRow(marker, samples, deterministicEvidence) {
  return (
    `${marker} entries=${WORKLOAD_SIZE} iterations_per_sample=${ITERATIONS_PER_SAMPLE} sample_pairs=${SAMPLE_PAIRS} ` +
    `percentile=nearest_rank pair_order=alternating_legacy_even legacy_ns=${samples.legacy.join(",")} ` +
    `optimized_ns=${samples.optimized.join(",")} legacy_p50_ns=${nearestRank(samples.legacy, 50)} ` +
    `legacy_p95_ns=${nearestRank(samples.legacy, 95)} optimized_p50_ns=${nearestRank(samples.optimized, 50)} ` +
    `optimized_p95_ns=${nearestRank(samples.optimized, 95)} ${deterministicEvidence}`
  );
}
