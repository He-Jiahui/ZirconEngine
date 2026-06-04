import type { HubShellState } from "../types/hub";
import brandMarkAsset from "../../../assets/brand/zircon-mark.svg";
import elysiumCover from "../../../assets/covers/reference/project-elysium.png";
import neonCover from "../../../assets/covers/reference/project-neon-streets.png";
import sandsCover from "../../../assets/covers/reference/project-sands-of-time.png";
import stellarCover from "../../../assets/covers/reference/project-stellar-outpost.png";
import woodsCover from "../../../assets/covers/reference/project-whispering-woods.png";

export const brandMark = brandMarkAsset;

export const coverById: Record<string, string> = {
  elysium: elysiumCover,
  "stellar-outpost": stellarCover,
  "sands-of-time": sandsCover,
  "whispering-woods": woodsCover,
  "neon-streets": neonCover,
};

export const fallbackShellState: HubShellState = {
  productName: "Zircon Hub",
  engineVersion: "Zircon Engine 1.8.2",
  activePage: "projects",
  taskStatus: [
    { id: "running", label: "Running", tone: "running" },
    { id: "success", label: "Success", tone: "success" },
    { id: "warning", label: "Warning", tone: "warning" },
    { id: "error", label: "Error", tone: "error" },
  ],
  projects: [
    {
      id: "elysium",
      name: "Elysium Chronicles",
      path: "C:\\ZirconProjects\\Elysium",
      modified: "Modified 2h ago",
      engineVersion: "1.8.2",
      platform: "Windows",
      coverId: "elysium",
    },
    {
      id: "stellar-outpost",
      name: "Stellar Outpost",
      path: "C:\\ZirconProjects\\StellarOutpost",
      modified: "Modified yesterday",
      engineVersion: "1.8.2",
      platform: "Windows",
      coverId: "stellar-outpost",
    },
    {
      id: "sands-of-time",
      name: "Sands of Time",
      path: "C:\\ZirconProjects\\SandsOfTime",
      modified: "Modified 3d ago",
      engineVersion: "1.8.1",
      platform: "Linux",
      coverId: "sands-of-time",
    },
    {
      id: "whispering-woods",
      name: "Whispering Woods",
      path: "C:\\ZirconProjects\\WhisperingWoods",
      modified: "Modified 1w ago",
      engineVersion: "1.8.0",
      platform: "Windows",
      coverId: "whispering-woods",
    },
  ],
  recentProjects: [
    {
      id: "elysium",
      name: "Elysium Chronicles",
      engineVersion: "1.8.2",
      modified: "2h ago",
      location: "C:\\ZirconProjects\\Elysium",
      coverId: "elysium",
    },
    {
      id: "stellar-outpost",
      name: "Stellar Outpost",
      engineVersion: "1.8.2",
      modified: "Yesterday",
      location: "C:\\ZirconProjects\\StellarOutpost",
      coverId: "stellar-outpost",
    },
    {
      id: "sands-of-time",
      name: "Sands of Time",
      engineVersion: "1.8.1",
      modified: "3d ago",
      location: "C:\\ZirconProjects\\SandsOfTime",
      coverId: "sands-of-time",
    },
    {
      id: "whispering-woods",
      name: "Whispering Woods",
      engineVersion: "1.8.0",
      modified: "1w ago",
      location: "C:\\ZirconProjects\\WhisperingWoods",
      coverId: "whispering-woods",
    },
    {
      id: "neon-streets",
      name: "Neon Streets",
      engineVersion: "1.7.9",
      modified: "2w ago",
      location: "C:\\ZirconProjects\\NeonStreets",
      coverId: "neon-streets",
    },
  ],
  quickActions: [
    {
      id: "build-project",
      title: "Build Project",
      detail: "Build your project for development or release",
      icon: "build",
    },
    {
      id: "install-device",
      title: "Install to Device",
      detail: "Deploy your project to a connected device",
      icon: "device",
    },
    {
      id: "package-project",
      title: "Package Project",
      detail: "Create a distributable package",
      icon: "package",
    },
    {
      id: "open-editor",
      title: "Open in Editor",
      detail: "Launch the editor with a project",
      icon: "editor",
    },
  ],
};
