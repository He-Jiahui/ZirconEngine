import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    check: { type: "boolean" },
    input: { type: "string", short: "i" },
  },
  strict: true,
});

if (!values.input) {
  throw new Error(
    "usage: node trace_hidden_symbol_codegen.mjs --input <full-trace-directory> [--check]",
  );
}

const projectRoot = path.resolve(import.meta.dirname, "..");
const inputRoot = path.resolve(values.input);
const outputPath = path.resolve(projectRoot, "reference", "trace_hidden_symbols.json");
const base = JSON.parse(
  fs.readFileSync(path.resolve(projectRoot, "reference", "trace_symbols.json"), "utf8"),
);
const goldenSymbolCount = base.golden_symbol_count ?? base.entries.length;
const baseSymbols = new Set(
  base.entries.slice(0, goldenSymbolCount).map((entry) => entry.text),
);
const files = fs.readdirSync(inputRoot).filter((name) => name.endsWith(".full.json")).sort();
assert.equal(files.length, 51, `expected 51 full reference traces, found ${files.length}`);

const hidden = new Map();
for (const file of files) {
  collect(JSON.parse(fs.readFileSync(path.resolve(inputRoot, file), "utf8")), hidden);
}
for (const text of baseSymbols) hidden.delete(text);

const entries = [...hidden]
  .sort(([left], [right]) => left.localeCompare(right))
  .map(([text, kinds]) => ({ text, kinds: [...kinds].sort() }));
const output = `${JSON.stringify(
  {
    schema_version: 1,
    source_commit: "7c10f280eec380e9877e66ce16333089e171fe42",
    full_trace_files: files.length,
    symbol_count: entries.length,
    entries,
  },
  null,
  2,
)}\n`;

if (values.check) {
  assert.equal(fs.readFileSync(outputPath, "utf8"), output, `${outputPath} is stale`);
  process.stdout.write(`checked ${entries.length} hidden trace symbols\n`);
} else {
  fs.writeFileSync(outputPath, output, "utf8");
  process.stdout.write(`generated ${entries.length} hidden trace symbols\n`);
}

function collect(value, symbols) {
  if (Array.isArray(value)) {
    for (const entry of value) collect(entry, symbols);
    return;
  }
  if (value && typeof value === "object") {
    for (const [key, entry] of Object.entries(value)) {
      add(symbols, key, "key");
      collect(entry, symbols);
    }
    return;
  }
  if (
    typeof value === "string" &&
    !/^[0-9a-f]{8}$/u.test(value) &&
    value !== "Infinity" &&
    value !== "-Infinity" &&
    value !== "NaN"
  ) {
    add(symbols, value, "value");
  }
}

function add(symbols, text, kind) {
  const kinds = symbols.get(text) ?? new Set();
  kinds.add(kind);
  symbols.set(text, kinds);
}
