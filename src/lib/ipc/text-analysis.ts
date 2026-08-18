import { invoke } from '@tauri-apps/api/core';
import type { TextStats } from '../domain/text-stats';
import type { TemplateVariable } from '../domain/template';

export async function computeTextStats(text: string): Promise<TextStats> {
  return invoke('compute_text_stats', { text });
}

export async function parseTemplate(templateText: string): Promise<{ rawText: string; variables: TemplateVariable[] }> {
  return invoke('parse_template', { templateText });
}

export async function renderTemplate(templateText: string, variablesJson: string): Promise<string> {
  return invoke('render_template', { templateText, variablesJson });
}
