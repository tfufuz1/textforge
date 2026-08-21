export type AppView = 'snippets' | 'clipboard' | 'scripts' | 'pipelines' | 'settings';

export interface WorkspaceSession {
  readonly activeView: AppView;
  readonly selectedSnippetId?: string;
  readonly selectedScriptId?: string;
  readonly selectedPipelineId?: string;
  readonly selectedClipboardId?: string;
  readonly searchQuery?: string;
  readonly sidebarOpen: boolean;
  readonly sidebarWidth?: number;
  readonly previewMode?: 'edit' | 'split' | 'preview';
  readonly filterState?: Record<string, unknown>;
  readonly savedAt?: number;
}

export function createDefaultSession(): WorkspaceSession {
  return {
    activeView: 'clipboard',
    sidebarOpen: true,
    sidebarWidth: 256,
    previewMode: 'split',
  };
}

export function patchWorkspaceSession(
  current: WorkspaceSession,
  patch: Partial<WorkspaceSession>
): WorkspaceSession {
  return {
    ...current,
    ...patch,
    savedAt: Date.now(),
  };
}
