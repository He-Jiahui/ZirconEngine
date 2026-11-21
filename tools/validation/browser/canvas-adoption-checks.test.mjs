import assert from 'node:assert/strict';
import test from 'node:test';
import { canvasAdoptionStage, canvasFollowupSubmitted } from './canvas-adoption-checks.mjs';

const contextId = 4;
const format = 'bgra8unorm';
const firstSize = { width: 64, height: 48 };
const resizedSize = { width: 96, height: 64 };

function observation() {
  return {
    fixtureError: null, interruption: null,
    gpu: { adapters: [{}], devices: [{}] },
    configure: [{ phase: 'load' }, { phase: 'load' }],
    canvas: { initial: firstSize }, resize: { requested: resizedSize },
    checks: {
      adapterReturned: true, deviceReturned: true, contextBound: true, initialConfigured: true,
      viewportHandoff: true, loaded: true, liveOwners: true, closed: true, ownersReleased: true,
    },
    frames: ['first-frame', 'resized-frame'].map(phase => ({ phase, receipt: { presented: true } })),
    submissions: [['first-frame', firstSize], ['resized-frame', resizedSize]].map(([phase, size]) => ({
      phase, returned: 1, error: null,
      canvasPasses: [{ texture: { phase, contextId, format, sameCanvas: true, ...size } }],
    })),
  };
}

function stage(report) {
  report.checks.followupSubmitted = canvasFollowupSubmitted(report, contextId, format);
  return canvasAdoptionStage(report);
}

test('both sizes need a presented receipt and a returned submission to the original Canvas', () => {
  assert.equal(stage(observation()), 'adopted');
});

test('loading, closing and reporting frames cannot substitute for Canvas submission', () => {
  const report = observation();
  report.submissions[0].canvasPasses = [];
  assert.equal(stage(report), 'frame-submission-incomplete');
});

test('the resized frame cannot borrow an attachment from another Canvas or phase', () => {
  for (const bad of [{ contextId: 8 }, { sameCanvas: false }, { phase: 'first-frame' },
    { format: 'rgba8unorm' }, firstSize]) {
    const report = observation();
    Object.assign(report.submissions[1].canvasPasses[0].texture, bad);
    assert.equal(stage(report), 'frame-submission-incomplete');
  }
});

test('a rejected submit or unpresented receipt is incomplete', () => {
  const rejected = observation();
  rejected.submissions[1].returned = null;
  assert.equal(stage(rejected), 'frame-submission-incomplete');
  const unpresented = observation();
  unpresented.frames[1].receipt.presented = false;
  assert.equal(stage(unpresented), 'frame-submission-incomplete');
});

test('a trap after initial configuration is distinct from viewport handoff or presentation', () => {
  const report = observation();
  report.interruption = { phase: 'load' };
  report.checks.viewportHandoff = false;
  report.checks.loaded = null;
  report.checks.ownersReleased = null;
  report.frames = [];
  report.submissions = [];
  assert.equal(stage(report), 'load-interrupted-after-first-configure');
});

test('valid frames still require owner lifecycle completion', () => {
  const report = observation();
  report.checks.ownersReleased = false;
  assert.equal(stage(report), 'owner-lifecycle-incomplete');
});
