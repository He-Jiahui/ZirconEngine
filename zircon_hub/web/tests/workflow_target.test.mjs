import assert from "node:assert/strict";
import test from "node:test";

import { workflowProjectTargetPayload, workflowTargetProject } from "../src/tauri/projectTarget.ts";

const recentProject = {
  id: "recent-project",
  name: "Recent project",
  location: "E:/projects/recent",
};

function state(selectedProject) {
  return { selectedProject, recentProjects: [recentProject] };
}

test("workflow commands require an explicitly selected existing project", () => {
  assert.equal(workflowTargetProject(state(null)), undefined);
  assert.equal(workflowProjectTargetPayload(state(null)), undefined);

  const missingProject = { id: "missing-project", path: "E:/projects/missing", exists: false };
  assert.equal(workflowTargetProject(state(missingProject)), missingProject);
  assert.equal(workflowProjectTargetPayload(state(missingProject)), undefined);

  const selectedProject = { id: "selected-project", path: "E:/projects/selected", exists: true };
  assert.equal(workflowTargetProject(state(selectedProject)), selectedProject);
  assert.deepEqual(workflowProjectTargetPayload(state(selectedProject)), {
    projectId: "selected-project",
    projectPath: "E:/projects/selected",
  });
});
