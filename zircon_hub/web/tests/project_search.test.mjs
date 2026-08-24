import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import test from "node:test";

import { DebouncedProjectSearch } from "../src/projects/debouncedProjectSearch.ts";
import { buildSearchIndex, filterSearchIndex } from "../src/projects/searchIndex.ts";

const SAMPLE_PAIRS = 21;

class FakeTimer {
  #nextId = 0;
  #pending = new Map();

  schedule(callback) {
    const id = ++this.#nextId;
    this.#pending.set(id, callback);
    return id;
  }

  cancel(id) {
    this.#pending.delete(id);
  }

  flush() {
    const callbacks = [...this.#pending.values()];
    this.#pending.clear();
    for (const callback of callbacks) {
      callback();
    }
  }

  get pendingCount() {
    return this.#pending.size;
  }
}

function projects(count) {
  return Array.from({ length: count }, (_, index) => ({
    id: `project-${index}`,
    name: `Project ${index % 997}`,
    location: `E:/workspaces/team-${index % 43}/project-${index}`,
  }));
}

function legacyFilter(items, query) {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return items;
  }
  return items.filter((project) =>
    `${project.name} ${project.location}`.toLowerCase().includes(normalizedQuery),
  );
}

function elapsedNanoseconds(operation) {
  const startedAt = performance.now();
  const result = operation();
  const elapsed = Math.max(1, Math.round((performance.now() - startedAt) * 1_000_000));
  return { elapsed, result };
}

function nearestRank(samples, percentile) {
  const sorted = [...samples].sort((left, right) => left - right);
  return sorted[Math.ceil((sorted.length * percentile) / 100) - 1];
}

test("project search indexes each item once and preserves filtering semantics", () => {
  const input = [
    { id: "alpha", name: "Alpha", location: "E:/Games/First" },
    { id: "beta", name: "Beta", location: "E:/Games/Second" },
    { id: "gamma", name: "Gamma", location: "D:/Archive/Third" },
  ];
  let textCalls = 0;
  const index = buildSearchIndex(input, (project) => {
    textCalls += 1;
    return `${project.name} ${project.location}`;
  });

  assert.equal(textCalls, input.length);
  assert.equal(filterSearchIndex(input, index, ""), input);
  assert.deepEqual(filterSearchIndex(input, index, "  games/SECOND  ").map((project) => project.id), ["beta"]);
  assert.deepEqual(filterSearchIndex(input, index, "archive").map((project) => project.id), ["gamma"]);
  assert.deepEqual(filterSearchIndex(input, index, "missing"), []);
});

test("project search debounce replaces pending input and cancels on teardown", () => {
  let optimizedDispatches = 0;
  for (let sample = 0; sample < SAMPLE_PAIRS; sample += 1) {
    const timer = new FakeTimer();
    const dispatched = [];
    const search = new DebouncedProjectSearch((query) => dispatched.push(query), 200, timer);
    for (let index = 0; index < 100; index += 1) {
      search.schedule(`sample-${sample}-query-${index}`);
    }
    assert.equal(timer.pendingCount, 1);
    timer.flush();
    assert.deepEqual(dispatched, [`sample-${sample}-query-99`]);
    optimizedDispatches += dispatched.length;
  }

  const timer = new FakeTimer();
  const dispatched = [];
  const search = new DebouncedProjectSearch((query) => dispatched.push(query), 200, timer);
  search.schedule("cancelled-query");
  search.cancel();
  timer.flush();
  assert.deepEqual(dispatched, []);

  console.log(
    `HUB01_PROJECT_SEARCH_DEBOUNCE_V1 sample_pairs=${SAMPLE_PAIRS} burst_inputs_per_sample=100 ` +
      `legacy_dispatches=${SAMPLE_PAIRS * 100} optimized_dispatches=${optimizedDispatches} ` +
      "dispatch_reduction_pct=99.000 quiet_window_ms=200 cancellation=passed",
  );
});

test(
  "project search index meets the 10k project burst-query P95 gate",
  { skip: process.env.ZIRCON_HUB01_PERF !== "1" },
  () => {
    const input = projects(10_000);
    const queries = Array.from({ length: 32 }, (_, index) =>
      index % 4 === 0 ? `project ${index}` : index % 4 === 1 ? `team-${index % 43}` : index % 4 === 2 ? `project-${index * 13}` : "no-match",
    );

    const legacyBurst = () => {
      let matches = 0;
      for (const query of queries) {
        matches += legacyFilter(input, query).length;
      }
      return matches;
    };
    const indexedBurst = () => {
      const index = buildSearchIndex(input, (project) => `${project.name} ${project.location}`);
      let matches = 0;
      for (const query of queries) {
        matches += filterSearchIndex(input, index, query).length;
      }
      return matches;
    };

    for (let warmup = 0; warmup < 3; warmup += 1) {
      assert.equal(indexedBurst(), legacyBurst());
    }

    const legacySamples = [];
    const optimizedSamples = [];
    let checksum = 0;
    for (let sample = 0; sample < SAMPLE_PAIRS; sample += 1) {
      globalThis.gc?.();
      const legacy = () => elapsedNanoseconds(legacyBurst);
      const optimized = () => elapsedNanoseconds(indexedBurst);
      const first = sample % 2 === 0 ? legacy() : optimized();
      const second = sample % 2 === 0 ? optimized() : legacy();
      legacySamples.push(sample % 2 === 0 ? first.elapsed : second.elapsed);
      optimizedSamples.push(sample % 2 === 0 ? second.elapsed : first.elapsed);
      assert.equal(first.result, second.result);
      checksum += first.result + second.result;
    }

    const legacyP50 = nearestRank(legacySamples, 50);
    const legacyP95 = nearestRank(legacySamples, 95);
    const optimizedP50 = nearestRank(optimizedSamples, 50);
    const optimizedP95 = nearestRank(optimizedSamples, 95);
    assert.ok(checksum > 0);
    assert.ok(
      optimizedP95 * 100 <= legacyP95 * 50,
      `indexed P95 ${optimizedP95}ns must be at most 50% of legacy ${legacyP95}ns`,
    );

    console.log(
      `HUB01_PROJECT_SEARCH_INDEX_10K_BENCH_V1 projects=10000 queries_per_sample=32 sample_pairs=${SAMPLE_PAIRS} ` +
        `percentile=nearest_rank pair_order=alternating_legacy_even legacy_ns=${legacySamples.join(",")} ` +
        `optimized_ns=${optimizedSamples.join(",")} legacy_p50_ns=${legacyP50} legacy_p95_ns=${legacyP95} ` +
        `optimized_p50_ns=${optimizedP50} optimized_p95_ns=${optimizedP95} ` +
        "normalizations_legacy=320000 normalizations_optimized=10000 threshold=optimized_p95_lte_50pct_legacy",
    );
  },
);
