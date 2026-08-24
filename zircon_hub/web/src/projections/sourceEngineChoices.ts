import type { HubSourceEngineSummary } from "../types/hub";

const MAX_FALLBACK_ENGINES = 2;

export interface SourceEngineChoices {
  activeEngines: HubSourceEngineSummary[];
  fallbackEngines: HubSourceEngineSummary[];
}

export function selectSourceEngineChoices(
  engines: readonly HubSourceEngineSummary[],
  requestedActiveEngineId?: string | null,
): SourceEngineChoices {
  const activeEngineId =
    requestedActiveEngineId ?? engines.find((engine) => engine.active)?.id ?? engines[0]?.id;
  const activeEngines: HubSourceEngineSummary[] = [];
  const fallbackEngines: HubSourceEngineSummary[] = [];

  for (const engine of engines) {
    if (engine.id === activeEngineId) {
      activeEngines.push(engine);
    } else if (fallbackEngines.length < MAX_FALLBACK_ENGINES) {
      fallbackEngines.push(engine);
    }
  }
  return { activeEngines, fallbackEngines };
}
