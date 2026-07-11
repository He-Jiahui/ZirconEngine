import { readdirSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const output = resolve(root, "node_modules/.cache/zircon-control-tests");
rmSync(output, { recursive: true, force: true });
run(process.execPath, [resolve(root, "node_modules/typescript/bin/tsc"), "-p", resolve(root, "tsconfig.test.json")]);
writeFileSync(resolve(output, "package.json"), '{"type":"commonjs"}\n', "utf8");
const compiledRoot = resolve(output, "tools/session_coordinator/web/src");
const tests = readdirSync(resolve(compiledRoot, "__tests__"))
  .filter((name) => name.endsWith(".test.js"))
  .map((name) => resolve(compiledRoot, "__tests__", name));
run(process.execPath, ["--test", ...tests]);

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit", env: { ...process.env, NODE_PATH: resolve(root, "node_modules") } });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
