import type { ZuiDiagnostic } from './bridge/zui-document';
import type { RenderedLayoutAudit } from './penpot-render-layout';

export interface LayoutReviewCaseMessage {
  id: string;
  sourcePath: string;
  host:
    'editor' | 'plugin' | 'woc' | 'component' | 'toolbar' | 'theme' | 'fixture';
  viewport: { width: number; height: number };
  dpi: number;
  locale: string;
  state: string;
  /** Optional scroll offset combined with the interaction state for this scene. */
  scrollPosition?: 'start' | 'end';
  data: Record<string, unknown>;
  themeSourcePath?: string;
  reviewHost?: { path: string; sha256: string };
}

export type PluginUiMessage =
  | { type: 'ready' }
  | {
      type: 'import-zui';
      fileName: string;
      source: string;
      replaceSelection?: boolean;
      /**
       * Design-review-only materialization policy. Product imports keep the
       * default `full` policy; the visual harness may request `leaf` for a
       * very large retained shell so composite source mappings stay as
       * editable semantic boards while primitive instances remain native.
       */
      nativeComponentMode?: 'full' | 'leaf' | 'semantic';
    }
  | {
      /** Set by the design-only visual harness before the next import. */
      type: 'set-review-materialization';
      mode: 'full' | 'leaf' | 'semantic';
    }
  | { type: 'export-zui' }
  | { type: 'set-preview-selection'; selected: boolean }
  | { type: 'audit-preview-layout'; requestId?: string }
  | { type: 'set-review-case'; reviewCase: LayoutReviewCaseMessage }
  | { type: 'inspect-selection' };

export type PluginHostMessage =
  | { type: 'theme'; content: string }
  | {
      type: 'status';
      level: 'idle' | 'working' | 'success' | 'warning' | 'error';
      message: string;
      diagnostics?: ZuiDiagnostic[];
      previewBoardId?: string;
      layoutAudit?: RenderedLayoutAudit;
      reviewCase?: LayoutReviewCaseMessage;
    }
  | { type: 'selection'; canExport: boolean; label: string | null }
  | {
      type: 'preview-audit';
      requestId?: string;
      previewBoardId: string;
      layoutAudit: RenderedLayoutAudit;
    }
  | {
      type: 'export-ready';
      fileName: string;
      source: string;
      diagnostics: ZuiDiagnostic[];
    };
