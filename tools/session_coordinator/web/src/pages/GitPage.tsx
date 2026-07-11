import type { GitProjection } from "../api/contracts";
import { MilestoneCommitEvidence } from "../components/git/MilestoneCommitEvidence";
import { HubPanel } from "../theme";
export function GitPage({ git }: { git: GitProjection }) { return <HubPanel title="里程碑最终提交证据"><MilestoneCommitEvidence requests={git.finalizeRequests} /></HubPanel>; }
