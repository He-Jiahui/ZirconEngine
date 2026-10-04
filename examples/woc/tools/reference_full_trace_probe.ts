// 调用端：tsx reference_full_trace_probe.ts --out <directory> [--all | scenarios]；职责：临时提升参考录制的采样帧为完整帧，并核对其投影后的 golden 形状。
// 请求的场景核对并写入后，在 finally 中恢复录制器原型。

import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseArgs } from 'node:util';
// BUG: [CR-M16-WOC-TOOLS-0001] 此处从实时检出目录导入场景，未绑定 golden 的固定提交；外来修改会改变探测执行。
import { Recorder, record, SCENARIOS, type Trace } from '../../../dev/world-of-claudecraft/tests/parity/index.ts';

const { values, positionals } = parseArgs({
  options: {
    all: { type: 'boolean' },
    out: { type: 'string', short: 'o' },
  },
  allowPositionals: true,
  strict: true,
});

if (!values.out || (!values.all && positionals.length === 0)) {
  throw new Error(
    'usage: tsx reference_full_trace_probe.ts --out <directory> [--all | <scenario>...]',
  );
}

const outputRoot = resolve(values.out);
const projectRoot = resolve(import.meta.dirname, '..');
mkdirSync(outputRoot, { recursive: true });

// 只在本次探测期间改写录制器，使每个采样帧都包含玩家和实体状态。
const prototype = Recorder.prototype as unknown as {
  pushFrame(label?: string, full?: boolean): void;
};
const originalPushFrame = prototype.pushFrame;
prototype.pushFrame = function pushEveryFullFrame(label?: string): void {
  originalPushFrame.call(this, label, true);
};

try {
  const requested = values.all ? SCENARIOS.map((scenario) => scenario.name) : positionals;
  for (const name of requested) {
    const scenario = SCENARIOS.find((candidate) => candidate.name === name);
    assert.ok(scenario, `unknown reference scenario ${name}`);
    const fullTrace = record(scenario).trace;
    const goldenPath = resolve(projectRoot, 'tests', 'parity', 'golden', `${name}.json`);
    const golden = JSON.parse(readFileSync(goldenPath, 'utf8')) as Trace;
    const projected = projectGoldenShape(fullTrace, golden);
    assert.deepStrictEqual(projected, golden, `${name} reference recorder drifted from golden`);
    assert.ok(
      fullTrace.frames.every((frame) => frame.players !== undefined && frame.entities !== undefined),
      `${name} probe omitted a full frame`,
    );
    writeFileSync(
      resolve(outputRoot, `${name}.full.json`),
      `${JSON.stringify(fullTrace)}\n`,
      'utf8',
    );
    process.stdout.write(`${name} ${fullTrace.frames.length} full frames exact\n`);
  }
} finally {
  prototype.pushFrame = originalPushFrame;
}

// 比较前移除签入的 golden 中不存在的字段，使完整录制匹配其预期形状。
function projectGoldenShape(fullTrace: Trace, golden: Trace): Trace {
  const trace = structuredClone(fullTrace);
  for (let index = 0; index < trace.frames.length; index += 1) {
    if (golden.frames[index]?.players === undefined) delete trace.frames[index].players;
    if (golden.frames[index]?.entities === undefined) delete trace.frames[index].entities;
  }
  return trace;
}
