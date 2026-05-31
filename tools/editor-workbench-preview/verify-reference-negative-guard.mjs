import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const sourceReference = resolve(rootDir, "docs/ui-and-layout/workbench.png");
const runtimeReference = resolve(rootDir, "zircon_editor/assets/ui/editor/reference/workbench.png");
const mismatchReference = resolve(
  rootDir,
  "docs/ui-and-layout/editor-workbench-designs/scene-workbench.png",
);

let negativePassed = false;
let ignoredOverridePassed = false;

const ignoredOverride = runVerifier({
  ...cleanVerifierEnv(),
  ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE: mismatchReference,
});
ignoredOverridePassed =
  ignoredOverride.status === 0 && readFileSync(sourceReference).equals(readFileSync(runtimeReference));

if (!ignoredOverridePassed) {
  console.error("Expected design:verify to ignore the runtime reference override without the guard flag.");
  console.error(`${ignoredOverride.stdout}\n${ignoredOverride.stderr}`.trim());
}

const negative = runVerifier({
  ...cleanVerifierEnv(),
  ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD: "1",
  ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE: mismatchReference,
});
const output = `${negative.stdout}\n${negative.stderr}`;
negativePassed = negative.status !== 0 && /not byte-identical|SHA-256|byte length/.test(output);

if (!negativePassed) {
  console.error("Expected design:verify to fail with the runtime reference override.");
  console.error(output.trim());
}

const restore = runVerifier(cleanVerifierEnv());
const restorePassed =
  restore.status === 0 && readFileSync(sourceReference).equals(readFileSync(runtimeReference));
if (!restorePassed) {
  console.error("The byte-identical runtime reference check failed after the negative guard.");
  console.error(`${restore.stdout}\n${restore.stderr}`.trim());
}

if (!ignoredOverridePassed || !negativePassed || !restorePassed) {
  process.exit(1);
}

console.log(
  "Verified guarded runtime reference override and restored byte-identical runtime reference.",
);

function runVerifier(env) {
  return spawnSync(process.execPath, ["verify-designs.mjs"], {
    cwd: new URL(".", import.meta.url),
    encoding: "utf8",
    env,
  });
}

function cleanVerifierEnv() {
  const env = { ...process.env };
  delete env.ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD;
  delete env.ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE;
  return env;
}
