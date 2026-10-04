import type { FrameLocator } from 'playwright';
import { requestPreviewLayoutAudit } from '../tools/penpot-preview-layout-audit';
it('uses the requested audit budget for iframe body resolution as well as the reply', async () => {
  const budget = 1234;
  const frame = {
    locator: () => ({
      evaluate: async (
        _callback: unknown,
        request: { requestId: string },
        options?: { timeout: number },
      ) => {
        if (options?.timeout !== budget)
          throw new Error(
            'Iframe resolution used an implicit shorter deadline',
          );
        return {
          requestId: request.requestId,
          previewBoardId: 'asset',
          layoutAudit: { overflowNodeIds: [], invalidGeometryNodeIds: [] },
        };
      },
    }),
  } as unknown as FrameLocator;
  await expect(requestPreviewLayoutAudit(frame, budget)).resolves.toMatchObject(
    { previewBoardId: 'asset' },
  );
});
