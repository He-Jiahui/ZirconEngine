// Each node opens layout and text work in the official frontend. Let that work
// drain between small batches instead of accumulating it for the whole asset.
const IMPORT_NODES_PER_TURN = 16;

export function createImportCheckpoint(): () => Promise<void> {
  let nodesThisTurn = 0;
  return async () => {
    nodesThisTurn += 1;
    if (nodesThisTurn < IMPORT_NODES_PER_TURN) return;
    nodesThisTurn = 0;
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
  };
}
