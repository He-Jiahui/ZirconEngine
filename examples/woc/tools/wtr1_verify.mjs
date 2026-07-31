import assert from "node:assert/strict";
import fs from "node:fs";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    golden: { type: "string", short: "g" },
    input: { type: "string", short: "i" },
  },
  strict: true,
});

if (!values.golden) {
  throw new Error(
    "usage: node wtr1_verify.mjs --golden <golden.json> [--input <trace.wtr1>]",
  );
}

const dictionary = JSON.parse(
  fs.readFileSync(new URL("../reference/trace_symbols.json", import.meta.url), "utf8"),
);
const symbols = new Map(dictionary.entries.map((entry) => [entry.id, entry.text]));
const expectedFingerprint = BigInt(`0x${dictionary.wire_fingerprint_hex}`);
let bytes;
if (values.input) {
  bytes = fs.readFileSync(values.input);
} else {
  const input = fs.readFileSync(0, "utf8");
  const hexLines = input.match(/^[0-9a-f]+$/gim) ?? [];
  const hex = hexLines.sort((left, right) => right.length - left.length)[0];
  if (!hex || hex.length % 2 !== 0) {
    throw new Error("stdin does not contain an even-length WTR1 hex line");
  }
  bytes = Buffer.from(hex, "hex");
}
let offset = 0;
let currentFrame = -1;

function take(length, context) {
  if (offset + length > bytes.length) {
    throw new Error(`truncated ${context} at ${offset}`);
  }
  const start = offset;
  offset += length;
  return bytes.subarray(start, offset);
}

function u8(context) {
  return take(1, context)[0];
}

function u16(context) {
  return take(2, context).readUInt16LE();
}

function u32(context) {
  return take(4, context).readUInt32LE();
}

function u64(context) {
  return take(8, context).readBigUInt64LE();
}

function safeNumber(value, context) {
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new Error(`${context} exceeds the JavaScript safe integer range`);
  }
  return Number(value);
}

function symbol(context) {
  const id = u16(context);
  const value = symbols.get(id);
  if (value === undefined) {
    throw new Error(`unknown ${context} symbol ${id}`);
  }
  return value;
}

function value(depth = 0) {
  if (depth > 64) {
    throw new Error("WTR1 value nesting exceeds 64");
  }
  const tag = u8("value tag");
  if (tag === 0) return null;
  if (tag === 1) return false;
  if (tag === 2) return true;
  if (tag === 3) return safeNumber(u64("unsigned value"), "unsigned value");
  if (tag === 4) {
    const negative = u8("signed value sign") === 1;
    const magnitude = safeNumber(u64("signed value magnitude"), "signed value");
    return negative && magnitude !== 0 ? -magnitude : magnitude;
  }
  if (tag === 5) {
    const negative = u8("fixed6 sign") === 1;
    const magnitude = safeNumber(u64("fixed6 magnitude"), "fixed6 value") / 1_000_000;
    return negative && magnitude !== 0 ? -magnitude : magnitude;
  }
  if (tag === 6) return Infinity;
  if (tag === 7) return -Infinity;
  if (tag === 8) return NaN;
  if (tag === 9) return symbol("string");
  if (tag === 10) {
    const length = u32("array length");
    return Array.from({ length }, () => value(depth + 1));
  }
  if (tag === 11) {
    const length = u32("object length");
    const result = {};
    for (let index = 0; index < length; index += 1) {
      const key = symbol("object key");
      if (Object.hasOwn(result, key)) {
        throw new Error(`duplicate WTR1 object key ${key}`);
      }
      result[key] = value(depth + 1);
    }
    return result;
  }
  throw new Error(`unknown WTR1 value tag ${tag} in frame ${currentFrame} at byte ${offset - 1}`);
}

function round6(number) {
  if (Number.isNaN(number)) return "NaN";
  if (number === Infinity) return "Infinity";
  if (number === -Infinity) return "-Infinity";
  if (Number.isInteger(number)) return number;
  return Math.round(number * 1_000_000) / 1_000_000;
}

function isInert(value) {
  if (value === null || value === undefined || value === 0 || value === false || value === "") {
    return true;
  }
  return Array.isArray(value) && value.length === 0;
}

function canonical(inputValue, omitDefaults = true) {
  if (inputValue === null || inputValue === undefined) return null;
  if (typeof inputValue === "number") return round6(inputValue);
  if (typeof inputValue === "string" || typeof inputValue === "boolean") return inputValue;
  if (Array.isArray(inputValue)) {
    return inputValue.map((entry) => canonical(entry, omitDefaults));
  }
  const result = {};
  for (const key of Object.keys(inputValue).sort()) {
    const entry = canonical(inputValue[key], omitDefaults);
    if (!omitDefaults || !isInert(entry)) result[key] = entry;
  }
  return result;
}

function fnv1a(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function digest(inputValue) {
  return fnv1a(JSON.stringify(canonical(inputValue, false)));
}

function hex32(number) {
  return number.toString(16).padStart(8, "0");
}

assert.equal(take(4, "magic").toString("ascii"), "WTR1");
assert.equal(u16("version"), 1);
assert.equal(u64("dictionary fingerprint"), expectedFingerprint);

const trace = {
  scenario: symbol("scenario"),
  seed: u32("seed"),
  sampleEvery: u32("sampleEvery"),
  ticks: u32("ticks"),
};
const coverageCount = u16("coverage count");
trace.coverage = Array.from({ length: coverageCount }, () => symbol("coverage"));
trace.draws = u32("draws");
trace.drawDigest = hex32(u32("drawDigest"));
const frameCount = u16("frame count");
trace.frames = [];

for (let index = 0; index < frameCount; index += 1) {
  currentFrame = index;
  const tick = safeNumber(u64("frame tick"), "frame tick");
  const time = canonical(value(), false);
  const nextId = safeNumber(u64("frame nextId"), "frame nextId");
  const labelId = u16("frame label");
  const full = u8("frame full") === 1;
  const rngDraws = u32("frame rng draws");
  const rngDigest = hex32(u32("frame rng digest"));
  const players = canonical(value(), true);
  const entities = canonical(value(), true);
  const eventDigest = hex32(u32("frame event digest"));
  const frame = {
    tick,
    time,
    nextId,
    state: digest({ players, entities }),
    events: eventDigest,
    rng: { draws: rngDraws, digest: rngDigest },
  };
  if (labelId !== 0) {
    const label = symbols.get(labelId);
    if (label === undefined) throw new Error(`unknown frame label symbol ${labelId}`);
    frame.label = label;
  }
  if (full) {
    frame.players = players;
    frame.entities = entities;
  }
  trace.frames.push(frame);
}

if (offset !== bytes.length) {
  throw new Error(`WTR1 has ${bytes.length - offset} trailing bytes`);
}

const golden = JSON.parse(fs.readFileSync(values.golden, "utf8"));
assert.deepStrictEqual(trace, golden);
console.log(
  JSON.stringify({
    bytes: bytes.length,
    byteDigest: fnv1a(bytes.toString("latin1")),
    scenario: trace.scenario,
    frames: trace.frames.length,
    finalState: trace.frames.at(-1).state,
    parity: "exact",
  }),
);
