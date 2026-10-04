import type { HubSourceEngineSummary } from "../types/hub";

const MAX_FALLBACK_ENGINES = 2;

export interface SourceEngineChoices {
  activeEngines: HubSourceEngineSummary[];
  fallbackEngines: HubSourceEngineSummary[];
}

function preferredSourceEngineId(
  engines: readonly HubSourceEngineSummary[],
  requestedActiveEngineId?: string | null,
): string | undefined {
  return requestedActiveEngineId ?? engines.find((engine) => engine.active)?.id;
}

export function admittedSourceEngineId(
  engines: readonly HubSourceEngineSummary[],
  requestedActiveEngineId?: string | null,
): string | undefined {
  const candidate = preferredSourceEngineId(engines, requestedActiveEngineId);
  if (!candidate) {
    return undefined;
  }
  return engines.some((engine) => engine.id === candidate)
    ? candidate
    : undefined;
}

export function selectSourceEngineChoices(
  engines: readonly HubSourceEngineSummary[],
  requestedActiveEngineId?: string | null,
): SourceEngineChoices {
  // Partitioning also proves admission; a separate existence scan would repeat
  // the full traversal when the active engine appears near the end of the list.
  const activeEngineId = preferredSourceEngineId(engines, requestedActiveEngineId);
  const activeEngines: HubSourceEngineSummary[] = [];
  const fallbackEngines: HubSourceEngineSummary[] = [];

  for (const engine of engines) {
    if (activeEngineId && engine.id === activeEngineId) {
      activeEngines.push(engine);
    } else if (fallbackEngines.length < MAX_FALLBACK_ENGINES) {
      fallbackEngines.push(engine);
    }
  }
  return { activeEngines, fallbackEngines };
}
