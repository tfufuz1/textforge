export type IgnoreRuleId = string & { readonly __brand: unique symbol };
export const IgnoreRuleId = {
  of: (raw: string): IgnoreRuleId => raw as IgnoreRuleId,
};

export type IgnoreMatchType = 'source_app' | 'content_regex' | 'content_type';

export interface ClipboardIgnoreRule {
  readonly id: IgnoreRuleId;
  readonly enabled: boolean;
  readonly matchType: IgnoreMatchType;
  readonly pattern: string;
  readonly createdAt: number;
}
