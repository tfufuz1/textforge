import { Option } from './adts';

export type AutomationRuleId = string & { readonly __brand: unique symbol };
export const AutomationRuleId = {
  of: (raw: string): AutomationRuleId => raw as AutomationRuleId,
};

export type AutomationTrigger =
  | { readonly _type: 'on_clipboard_change' }
  | { readonly _type: 'on_snippet_insert'; readonly snippetId?: string }
  | { readonly _type: 'on_app_focus'; readonly appPattern: string }
  | { readonly _type: 'on_content_pattern'; readonly regex: string };

export interface AutomationRule {
  readonly id: AutomationRuleId;
  readonly name: string;
  readonly enabled: boolean;
  readonly trigger: AutomationTrigger;
  readonly condition: Option<string>;
  readonly scriptId: string;
  readonly sortOrder: number;
  readonly createdAt: number;
  readonly updatedAt: number;
}
