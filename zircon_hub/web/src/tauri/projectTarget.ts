import type { HubProjectDetail, HubRecentProject, HubShellState, ProjectTargetPayload } from "../types/hub";

export function projectTargetPayload(project?: HubProjectDetail | null): ProjectTargetPayload | undefined {
  if (!project) {
    return undefined;
  }

  return {
    projectId: project.id,
    projectPath: project.path,
  };
}

export function workflowProjectTargetPayload(state: HubShellState): ProjectTargetPayload | undefined {
  const target = workflowTargetProject(state);
  if (!target) {
    return undefined;
  }

  return {
    projectId: target.id,
    projectPath: workflowProjectPath(target),
  };
}

export function workflowTargetProject(state: HubShellState): HubProjectDetail | HubRecentProject | undefined {
  return state.selectedProject ?? state.recentProjects[0];
}

export function workflowProjectPath(target: HubProjectDetail | HubRecentProject): string {
  return "path" in target ? target.path : target.location;
}

export function quickActionProjectTargetPayload(project?: HubProjectDetail | null): ProjectTargetPayload | undefined {
  if (!project?.exists) {
    return undefined;
  }

  return projectTargetPayload(project);
}
