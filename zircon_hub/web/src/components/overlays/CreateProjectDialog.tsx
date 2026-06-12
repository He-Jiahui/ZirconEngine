import { Box } from "@mui/material";
import { useEffect, useState } from "react";
import type {
  CreateProjectPayload,
  HubActionText,
  HubProjectsText,
  HubProjectTemplate,
  HubSourceEngineSummary,
} from "../../types/hub";
import { HubButton, HubComboBox, HubTextField } from "../inputs";
import { HubDialog } from "./HubDialog";

export interface CreateProjectDialogProps {
  open: boolean;
  templates: HubProjectTemplate[];
  sourceEngines: HubSourceEngineSummary[];
  activeSourceEngineId: string | null;
  defaultProjectDir: string;
  text: HubProjectsText;
  actionText: HubActionText;
  onClose: () => void;
  onCreate: (payload: CreateProjectPayload) => void;
}

export function CreateProjectDialog({
  open,
  templates,
  sourceEngines,
  activeSourceEngineId,
  defaultProjectDir,
  text,
  actionText,
  onClose,
  onCreate,
}: CreateProjectDialogProps) {
  const [projectName, setProjectName] = useState("");
  const [projectLocation, setProjectLocation] = useState(defaultProjectDir);
  const [template, setTemplate] = useState("renderable-empty");
  const [engineId, setEngineId] = useState(activeSourceEngineId ?? sourceEngines[0]?.id ?? "");

  useEffect(() => {
    setProjectLocation(defaultProjectDir);
  }, [defaultProjectDir]);

  useEffect(() => {
    setEngineId((currentEngineId) => {
      if (sourceEngines.some((engine) => engine.id === currentEngineId)) {
        return currentEngineId;
      }
      return activeSourceEngineId ?? sourceEngines[0]?.id ?? "";
    });
  }, [activeSourceEngineId, sourceEngines]);

  useEffect(() => {
    if (templates.some((projectTemplate) => projectTemplate.id === template && projectTemplate.enabled)) {
      return;
    }
    const firstEnabled = templates.find((projectTemplate) => projectTemplate.enabled);
    if (firstEnabled) {
      setTemplate(firstEnabled.id);
    }
  }, [templates, template]);

  const selectedTemplate = templates.find((projectTemplate) => projectTemplate.id === template);
  const createDisabled = projectName.trim().length === 0 || projectLocation.trim().length === 0 || !selectedTemplate?.enabled;

  const createProject = () => {
    if (createDisabled) {
      return;
    }
    onCreate({
      name: projectName,
      location: projectLocation,
      template,
      engineId: engineId || null,
    });
  };

  return (
    <HubDialog
      open={open}
      title={text.newProjectDialog}
      onClose={onClose}
      actions={
        <>
          <HubButton onClick={onClose}>{actionText.close}</HubButton>
          <HubButton tone="primary" disabled={createDisabled} onClick={createProject}>
            {actionText.createProject}
          </HubButton>
        </>
      }
    >
      <Box sx={{ display: "grid", gap: 1.4, pt: 0.5 }}>
        <HubTextField label={text.projectName} value={projectName} onChange={(event) => setProjectName(event.target.value)} />
        <HubTextField label={text.location} value={projectLocation} onChange={(event) => setProjectLocation(event.target.value)} />
        <HubComboBox
          value={engineId}
          minWidth={0}
          placeholder={text.sourceEngine}
          options={sourceEngines.map((engine) => ({
            value: engine.id,
            label: engine.name,
            detail: engine.sourcePath,
          }))}
          onChange={setEngineId}
        />
        <HubComboBox
          value={template}
          minWidth={0}
          placeholder={text.template}
          options={templates.map((projectTemplate) => ({
            value: projectTemplate.id,
            label: projectTemplate.optionLabel,
            detail: projectTemplate.disabledReason ?? projectTemplate.description,
            disabled: !projectTemplate.enabled,
          }))}
          onChange={setTemplate}
        />
      </Box>
    </HubDialog>
  );
}
