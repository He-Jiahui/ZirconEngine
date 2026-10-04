import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import test from "node:test";

import { DebouncedProjectSearch } from "../src/projects/debouncedProjectSearch.ts";
import { buildSearchIndex, filterSearchIndex } from "../src/projects/searchIndex.ts";

// 功能测试不依赖性能开关；专用性能运行使用配对样本比较同一批查询的预处理成本。
const SAMPLE_PAIRS = 21;

// 可控计时器让输入替换和卸载取消可确定地验证，不以真实等待时间推断生命周期是否正确。
class FakeTimer {
  #nextId = 0;
  #pending = new Map();

  // 返回唯一句柄，测试可统计尚未发布的输入数量；回调只在显式推进时运行。
  schedule(callback) {
    const id = ++this.#nextId;
    this.#pending.set(id, callback);
    return id;
  }

  // 取消只移除尚未推进的回调，模拟页面销毁前撤销待发布输入。
  cancel(id) {
    this.#pending.delete(id);
  }

  // 一次推进取出当前队列，回调中新调度的输入留给下一次推进，避免把不同静默窗口混为一批。
  flush() {
    const callbacks = [...this.#pending.values()];
    this.#pending.clear();
    for (const callback of callbacks) {
      callback();
    }
  }

  // 待发布数量用于检验连续输入是否收敛到一个请求，而不观察内部私有状态。
  get pendingCount() {
    return this.#pending.size;
  }
}

// 合成项目名和路径供大量查询复用，只衡量索引筛选，不包含文件系统或后端通信。
function projects(count) {
  return Array.from({ length: count }, (_, index) => ({
    id: `project-${index}`,
    name: `Project ${index % 997}`,
    location: `E:/workspaces/team-${index % 43}/project-${index}`,
  }));
}

// 保留旧搜索的空查询身份及文本组合语义，作为功能和性能比较的独立基线。
function legacyFilter(items, query) {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return items;
  }
  return items.filter((project) =>
    `${project.name} ${project.location}`.toLowerCase().includes(normalizedQuery),
  );
}

// 计时返回结果用于配对一致性检查，避免以少处理项目换取虚假的速度收益。
function elapsedNanoseconds(operation) {
  const startedAt = performance.now();
  const result = operation();
  const elapsed = Math.max(1, Math.round((performance.now() - startedAt) * 1_000_000));
  return { elapsed, result };
}

// 性能比较采用统一秩统计，调用方只传入非空的配对样本与本文件固定百分位。
function nearestRank(samples, percentile) {
  const sorted = [...samples].sort((left, right) => left - right);
  return sorted[Math.ceil((sorted.length * percentile) / 100) - 1];
}

// 同时约束一次建索引、空查询数组身份及原项目顺序，供两个项目页面共享搜索行为。
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

// 连续输入只发布最后一项，生命周期结束则不发布；此测试只覆盖调度器，不覆盖页面动作回包。
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

// 每轮突发查询纳入一次索引构建后再比较复用收益；页面实际按数据快照缓存索引，纯函数门槛不代表端到端延迟。
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
