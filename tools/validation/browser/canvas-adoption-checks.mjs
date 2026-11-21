// Evaluate observations only. The browser fixture obtains them from real WebGPU calls.
export function canvasFollowupSubmitted(report, contextId, format) {
  if (contextId == null || format == null) return false;
  const expected = [
    ['first-frame', report.canvas.initial],
    ['resized-frame', report.resize?.requested],
  ];
  return expected.every(([phase, size]) => size &&
    report.frames.some(frame => frame.phase === phase && !frame.error && frame.receipt?.presented === true) &&
    report.submissions.some(event => event.phase === phase && !event.error &&
      Number.isSafeInteger(event.returned) && event.canvasPasses.some(({ texture }) =>
        texture.phase === phase && texture.sameCanvas && texture.contextId === contextId &&
        texture.format === format && texture.width === size.width && texture.height === size.height)));
}

export function canvasAdoptionStage({ fixtureError, gpu, checks, configure, interruption }) {
  if (interruption) {
    const step = interruption.phase;
    if (step === 'load' && checks.initialConfigured) {
      return checks.viewportHandoff ? 'load-interrupted-after-viewport-handoff' : 'load-interrupted-after-first-configure';
    }
    return `${step}-interrupted`;
  }
  if (fixtureError) return 'fixture-error';
  if (!gpu.adapters.length) return 'before-adapter-request';
  if (!checks.adapterReturned) return 'adapter-unresolved';
  if (!gpu.devices.length) return 'before-device-request';
  if (!checks.deviceReturned) return 'device-unresolved';
  if (!configure.some(event => event.phase === 'load')) return 'before-first-configure';
  if (!checks.initialConfigured) return 'first-configure-incomplete';
  if (!checks.viewportHandoff) return 'before-viewport-handoff';
  if (!checks.loaded) return 'load-failed-after-viewport-handoff';
  if (!checks.followupSubmitted) return 'frame-submission-incomplete';
  if (!(checks.liveOwners && checks.closed && checks.ownersReleased)) return 'owner-lifecycle-incomplete';
  return 'adopted';
}
