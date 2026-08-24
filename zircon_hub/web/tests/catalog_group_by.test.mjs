import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import test from "node:test";

import { groupBy } from "../src/catalog/groupBy.ts";

const SAMPLE_PAIRS = 21;

function legacyGroupBy(items, key) {
  return items.reduce((groups, item) => {
    const groupKey = key(item);
    groups.set(groupKey, [...(groups.get(groupKey) ?? []), item]);
    return groups;
  }, new Map());
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

function rows(count, groupCount = 1) {
  return Array.from({ length: count }, (_, index) => ({
    id: index,
    category: `category-${index % groupCount}`,
  }));
}

function totalItems(groups) {
  return Array.from(groups.values()).reduce((total, group) => total + group.length, 0);
}

test("groupBy preserves first-key and item order while evaluating each key once", () => {
  const input = [
    { id: 1, category: "b" },
    { id: 2, category: "a" },
    { id: 3, category: "b" },
    { id: 4, category: "c" },
    { id: 5, category: "a" },
  ];
  let keyCalls = 0;

  const groups = groupBy(input, (item) => {
    keyCalls += 1;
    return item.category;
  });

  assert.equal(keyCalls, input.length);
  assert.deepEqual([...groups.keys()], ["b", "a", "c"]);
  assert.deepEqual(
    [...groups.values()].map((group) => group.map((item) => item.id)),
    [[1, 3], [2, 5], [4]],
  );
});

test("groupBy constructs a complete 100k single-group catalog", () => {
  const input = rows(100_000);
  const groups = groupBy(input, (item) => item.category);

  assert.equal(groups.size, 1);
  assert.equal(totalItems(groups), input.length);
  assert.equal(groups.get("category-0")?.[99_999]?.id, 99_999);
});

test(
  "groupBy meets the Hub02 10k comparison and 100k linear-scale gates",
  { skip: process.env.ZIRCON_HUB02_PERF !== "1" },
  () => {
    const tenThousand = rows(10_000);
    const hundredThousand = rows(100_000);

    for (let warmup = 0; warmup < 3; warmup += 1) {
      groupBy(tenThousand, (item) => item.category);
      groupBy(hundredThousand, (item) => item.category);
      legacyGroupBy(tenThousand, (item) => item.category);
    }

    const legacySamples = [];
    const optimizedSamples = [];
    const tenByTenSamples = [];
    const hundredThousandSamples = [];
    let checksum = 0;

    for (let sample = 0; sample < SAMPLE_PAIRS; sample += 1) {
      globalThis.gc?.();
      const measureLegacy = () => elapsedNanoseconds(() => legacyGroupBy(tenThousand, (item) => item.category));
      const measureOptimized = () => elapsedNanoseconds(() => groupBy(tenThousand, (item) => item.category));
      const first = sample % 2 === 0 ? measureLegacy() : measureOptimized();
      const second = sample % 2 === 0 ? measureOptimized() : measureLegacy();
      legacySamples.push(sample % 2 === 0 ? first.elapsed : second.elapsed);
      optimizedSamples.push(sample % 2 === 0 ? second.elapsed : first.elapsed);
      checksum += totalItems(first.result) + totalItems(second.result);

      globalThis.gc?.();
      const measureTenByTen = () =>
        elapsedNanoseconds(() => {
          let total = 0;
          for (let repetition = 0; repetition < 10; repetition += 1) {
            total += totalItems(groupBy(tenThousand, (item) => item.category));
          }
          return total;
        });
      const measureHundredThousand = () =>
        elapsedNanoseconds(() => totalItems(groupBy(hundredThousand, (item) => item.category)));
      const scaleFirst = sample % 2 === 0 ? measureTenByTen() : measureHundredThousand();
      const scaleSecond = sample % 2 === 0 ? measureHundredThousand() : measureTenByTen();
      tenByTenSamples.push(sample % 2 === 0 ? scaleFirst.elapsed : scaleSecond.elapsed);
      hundredThousandSamples.push(sample % 2 === 0 ? scaleSecond.elapsed : scaleFirst.elapsed);
      checksum += scaleFirst.result + scaleSecond.result;
    }

    const legacyP50 = nearestRank(legacySamples, 50);
    const legacyP95 = nearestRank(legacySamples, 95);
    const optimizedP50 = nearestRank(optimizedSamples, 50);
    const optimizedP95 = nearestRank(optimizedSamples, 95);
    const tenByTenP50 = nearestRank(tenByTenSamples, 50);
    const tenByTenP95 = nearestRank(tenByTenSamples, 95);
    const hundredThousandP50 = nearestRank(hundredThousandSamples, 50);
    const hundredThousandP95 = nearestRank(hundredThousandSamples, 95);

    assert.equal(checksum, SAMPLE_PAIRS * 220_000);
    assert.ok(
      optimizedP95 * 100 <= legacyP95 * 25,
      `10k optimized P95 ${optimizedP95}ns must be at most 25% of legacy ${legacyP95}ns`,
    );
    assert.ok(
      hundredThousandP95 * 100 <= tenByTenP95 * 500,
      `100k P95 ${hundredThousandP95}ns must be at most 500% of 10x10k ${tenByTenP95}ns`,
    );

    console.log(
      `HUB02_CATALOG_GROUP_BY_10K_BENCH_V1 entries=10000 groups=1 sample_pairs=${SAMPLE_PAIRS} ` +
        `percentile=nearest_rank pair_order=alternating_legacy_even legacy_ns=${legacySamples.join(",")} ` +
        `optimized_ns=${optimizedSamples.join(",")} legacy_p50_ns=${legacyP50} legacy_p95_ns=${legacyP95} ` +
        `optimized_p50_ns=${optimizedP50} optimized_p95_ns=${optimizedP95} ` +
        "legacy_reference_copies=50005000 optimized_appends=10000 legacy_map_lookups=10000 " +
        "optimized_map_lookups=1 optimized_consecutive_cache_hits=9999 " +
        "threshold=optimized_p95_lte_25pct_legacy",
    );
    console.log(
      `HUB02_CATALOG_GROUP_BY_100K_BENCH_V1 entries=100000 groups=1 sample_pairs=${SAMPLE_PAIRS} ` +
        `percentile=nearest_rank pair_order=alternating_ten_by_ten_even ten_by_ten_ns=${tenByTenSamples.join(",")} ` +
        `hundred_thousand_ns=${hundredThousandSamples.join(",")} ten_by_ten_p50_ns=${tenByTenP50} ` +
        `ten_by_ten_p95_ns=${tenByTenP95} hundred_thousand_p50_ns=${hundredThousandP50} ` +
        `hundred_thousand_p95_ns=${hundredThousandP95} optimized_appends=100000 ` +
        "threshold=hundred_thousand_p95_lte_500pct_ten_by_ten_p95",
    );
  },
);
