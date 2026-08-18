export type AppView = 'snippets' | 'clipboard' | 'scripts' | 'pipelines' | 'settings';

export interface WorkspaceSession {
  readonly activeView: AppView;
  readonly selectedSnippetId?: string;
  readonly selectedScriptId?: string;
  readonly selectedPipelineId?: string;
  readonly searchQuery?: string;
  readonly sidebarOpen: boolean;
}
