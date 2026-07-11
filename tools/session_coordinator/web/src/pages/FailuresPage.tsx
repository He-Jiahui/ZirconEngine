import type { FailureProjection } from "../api/contracts";
import { FailureGraph } from "../components/failure/FailureGraph";
import { HubPanel } from "../theme";
export function FailuresPage({ failures }: { failures: FailureProjection }) { return <HubPanel title="Failure 图级治理"><FailureGraph nodes={failures.nodes} diagnostics={failures.diagnostics} /></HubPanel>; }
