import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    full: { type: "string", short: "f" },
    golden: { type: "string", short: "g" },
    out: { type: "string", short: "o" },
  },
  strict: true,
});

if (!values.full || !values.golden || !values.out) {
  throw new Error(
    "usage: node wtr1_encode.mjs --full <full.json> --golden <golden.json> --out <trace.wtr1>",
  );
}

const dictionary = JSON.parse(
  fs.readFileSync(new URL("../reference/trace_symbols.json", import.meta.url), "utf8"),
);
const symbols = new Map(dictionary.entries.map((entry) => [entry.text, entry.id]));
const fingerprint = BigInt(`0x${dictionary.wire_fingerprint_hex}`);
const fullTrace = JSON.parse(fs.readFileSync(values.full, "utf8"));
const golden = JSON.parse(fs.readFileSync(values.golden, "utf8"));

assert.deepStrictEqual(projectGoldenShape(fullTrace, golden), golden);
assert.equal(fullTrace.frames.length, golden.frames.length);

const bytes = [];

raw(Buffer.from("WTR1", "ascii"));
u16(1);
u64(fingerprint);
symbol(fullTrace.scenario, "scenario");
u32(fullTrace.seed);
u32(fullTrace.sampleEvery);
u32(fullTrace.ticks);
u16(fullTrace.coverage.length);
for (const entry of fullTrace.coverage) symbol(entry, "coverage");
u32(fullTrace.draws);
u32(hex32(fullTrace.drawDigest, "draw digest"));
u16(fullTrace.frames.length);

for (let index = 0; index < fullTrace.frames.length; index += 1) {
  const frame = fullTrace.frames[index];
  const goldenFrame = golden.frames[index];
  assert.ok(Array.isArray(frame.players), `frame ${index} is missing players`);
  assert.ok(Array.isArray(frame.entities), `frame ${index} is missing entities`);
  assert.equal(
    digest({ players: frame.players, entities: frame.entities }),
    frame.state,
    `frame ${index} state digest drift`,
  );
  u64(BigInt(frame.tick));
  value(frame.time);
  u64(BigInt(frame.nextId));
  if (frame.label === undefined) u16(0);
  else symbol(frame.label, `frame ${index} label`);
  u8(goldenFrame.players === undefined ? 0 : 1);
  u32(frame.rng.draws);
  u32(hex32(frame.rng.digest, `frame ${index} RNG digest`));
  value(frame.players);
  value(frame.entities);
  u32(hex32(frame.events, `frame ${index} event digest`));
}

const output = Buffer.from(bytes);
fs.mkdirSync(path.dirname(path.resolve(values.out)), { recursive: true });
fs.writeFileSync(values.out, output);
process.stdout.write(
  `${JSON.stringify({
    scenario: fullTrace.scenario,
    bytes: output.length,
    byteDigest: fnv1a(output.toString("latin1")),
    frames: fullTrace.frames.length,
    finalState: fullTrace.frames.at(-1).state,
  })}\n`,
);

function projectGoldenShape(trace, expected) {
  const projected = structuredClone(trace);
  for (let index = 0; index < projected.frames.length; index += 1) {
    if (expected.frames[index]?.players === undefined) delete projected.frames[index].players;
    if (expected.frames[index]?.entities === undefined) delete projected.frames[index].entities;
  }
  return projected;
}

function raw(buffer) {
  for (const byte of buffer) bytes.push(byte);
}

function u8(number) {
  assert.ok(Number.isInteger(number) && number >= 0 && number <= 0xff);
  bytes.push(number);
}

function u16(number) {
  assert.ok(Number.isInteger(number) && number >= 0 && number <= 0xffff);
  u8(number & 0xff);
  u8((number >>> 8) & 0xff);
}

function u32(number) {
  assert.ok(Number.isInteger(number) && number >= 0 && number <= 0xffffffff);
  const value = number >>> 0;
  u16(value & 0xffff);
  u16(value >>> 16);
}

function u64(number) {
  assert.ok(number >= 0n && number <= 0xffffffffffffffffn);
  u32(Number(number & 0xffffffffn));
  u32(Number((number >> 32n) & 0xffffffffn));
}

function symbol(text, context) {
  const id = symbols.get(text);
  assert.ok(id !== undefined, `unknown ${context} symbol ${JSON.stringify(text)}`);
  u16(id);
}

function value(input, depth = 0) {
  assert.ok(depth <= 64, "WTR1 value nesting exceeds 64");
  if (input === null || input === undefined) {
    u8(0);
    return;
  }
  if (input === false) {
    u8(1);
    return;
  }
  if (input === true) {
    u8(2);
    return;
  }
  if (typeof input === "number") {
    assert.ok(Number.isSafeInteger(input) || Number.isFinite(input));
    if (Number.isInteger(input)) {
      if (input >= 0) {
        u8(3);
        u64(BigInt(input));
      } else {
        u8(4);
        u8(1);
        u64(BigInt(-input));
      }
      return;
    }
    u8(5);
    u8(input < 0 ? 1 : 0);
    u64(BigInt(Math.round(Math.abs(input) * 1_000_000)));
    return;
  }
  if (typeof input === "string") {
    if (input === "Infinity") u8(6);
    else if (input === "-Infinity") u8(7);
    else if (input === "NaN") u8(8);
    else {
      u8(9);
      symbol(input, "string value");
    }
    return;
  }
  if (Array.isArray(input)) {
    u8(10);
    u32(input.length);
    for (const entry of input) value(entry, depth + 1);
    return;
  }
  assert.equal(typeof input, "object");
  const keys = Object.keys(input).sort();
  u8(11);
  u32(keys.length);
  for (const key of keys) {
    symbol(key, "object key");
    value(input[key], depth + 1);
  }
}

function hex32(text, context) {
  assert.match(text, /^[0-9a-f]{8}$/u, `${context} is not an eight-digit hex value`);
  return Number.parseInt(text, 16) >>> 0;
}

function round6(number) {
  if (Number.isNaN(number)) return "NaN";
  if (number === Infinity) return "Infinity";
  if (number === -Infinity) return "-Infinity";
  if (Number.isInteger(number)) return number;
  return Math.round(number * 1_000_000) / 1_000_000;
}

function canonical(input, omitDefaults = true) {
  if (input === null || input === undefined) return null;
  if (typeof input === "number") return round6(input);
  if (typeof input === "string" || typeof input === "boolean") return input;
  if (Array.isArray(input)) return input.map((entry) => canonical(entry, omitDefaults));
  const result = {};
  for (const key of Object.keys(input).sort()) {
    const entry = canonical(input[key], omitDefaults);
    if (!omitDefaults || !isInert(entry)) result[key] = entry;
  }
  return result;
}

function isInert(input) {
  return input === null || input === undefined || input === 0 || input === false || input === "" ||
    (Array.isArray(input) && input.length === 0);
}

function digest(input) {
  return fnv1a(JSON.stringify(canonical(input, false)));
}

function fnv1a(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}
