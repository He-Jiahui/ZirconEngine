import test from "node:test";
import assert from "node:assert/strict";
import { JSDOM } from "jsdom";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { controlClient } from "../api/client";
import { useOverviewReportData } from "../components/dashboard/useOverviewReportData";

test("overview history request survives ordinary snapshot rerenders", async () => {
  const dom = new JSDOM("<!doctype html><div id='root'></div>", { url: "http://localhost/ui/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement });
  (globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

  const originals = {
    validationHistory: controlClient.validationHistory,
    failureHistory: controlClient.failureHistory,
    logs: controlClient.logs,
  };
  let validationCalls = 0;
  let validationSignal: AbortSignal | undefined;
  controlClient.validationHistory = async (_limit, signal) => {
    validationCalls += 1;
    validationSignal = signal;
    return new Promise(() => undefined);
  };
  controlClient.failureHistory = async () => new Promise(() => undefined);
  controlClient.logs = async () => new Promise(() => undefined);

  function Harness({ revision }: { revision: number }) {
    useOverviewReportData();
    return <span>{revision}</span>;
  }

  const host = document.querySelector("#root") as HTMLDivElement;
  const root = createRoot(host);
  try {
    await act(async () => { root.render(<Harness revision={1} />); await Promise.resolve(); });
    assert.equal(validationCalls, 1);
    assert.equal(validationSignal?.aborted, false);

    await act(async () => { root.render(<Harness revision={2} />); await Promise.resolve(); });
    assert.equal(validationCalls, 1);
    assert.equal(validationSignal?.aborted, false);

    await act(async () => root.unmount());
    assert.equal(validationSignal?.aborted, true);
  } finally {
    controlClient.validationHistory = originals.validationHistory;
    controlClient.failureHistory = originals.failureHistory;
    controlClient.logs = originals.logs;
    dom.window.close();
  }
});
