import type { ProjectionGeometry } from './penpot-projection-model';

export interface ControlPart extends ProjectionGeometry {
  fill: string;
  radius: number;
  stroke?: string;
  strokeWidth?: number;
}

export interface ControlGeometry {
  parts: Record<string, ControlPart>;
  texts: Record<string, ProjectionGeometry>;
}
