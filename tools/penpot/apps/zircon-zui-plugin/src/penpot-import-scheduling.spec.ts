import { describe, expect, it } from 'vitest';
import { createImportCheckpoint } from './penpot-import-scheduling';

describe('Penpot import scheduling', () => {
  it('lets queued host work run before a large import finishes', async () => {
    const checkpoint = createImportCheckpoint();
    let hostWorkComplete = false;
    const hostWork = new Promise<void>((resolve) => {
      setTimeout(() => {
        hostWorkComplete = true;
        resolve();
      }, 0);
    });
    let nodesBeforeHostWork = 0;
    for (let index = 0; index < 256; index += 1) {
      await checkpoint();
      if (!hostWorkComplete) nodesBeforeHostWork += 1;
    }
    expect(hostWorkComplete).toBe(true);
    expect(nodesBeforeHostWork).toBeGreaterThan(0);
    expect(nodesBeforeHostWork).toBeLessThan(256);
    await hostWork;
  });
});
