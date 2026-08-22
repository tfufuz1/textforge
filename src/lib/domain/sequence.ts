import { Result } from './adts';
import type { DomainError } from './errors';

export type SequenceId = string & { readonly __brand: unique symbol };
export const SequenceId = {
  of: (raw: string): SequenceId => raw as SequenceId,
};

export type SequenceSeparator =
  | { readonly _type: 'none' }
  | { readonly _type: 'newline'; readonly count: number }
  | { readonly _type: 'custom'; readonly text: string }
  | { readonly _type: 'numbered_list' }
  | { readonly _type: 'markdown_section' };

export type ItemRef =
  | { readonly _type: 'snippet'; readonly id: string }
  | { readonly _type: 'clipboard'; readonly id: string }
  | { readonly _type: 'script_output'; readonly scriptId: string }
  | { readonly _type: 'literal'; readonly text: string };

export interface SequenceItem {
  readonly id: string;
  readonly order: number;
  readonly ref: ItemRef;
  readonly pipelineId: string | null;
  readonly prefixOverride: string | null;
  readonly suffixOverride: string | null;
  readonly enabled: boolean;
}

export interface Sequence {
  readonly id: SequenceId;
  readonly name: string;
  readonly items: readonly SequenceItem[];
  readonly separator: SequenceSeparator;
  readonly tags: readonly string[];
  readonly favorite: boolean;
  readonly createdAt: number;
  readonly updatedAt: number;
  readonly lastRenderedAt: number | null;
}

export const Sequence = {
  create: (draft: {
    name: string;
    items?: readonly Omit<SequenceItem, 'order'>[];
    separator?: SequenceSeparator;
  }): Result<DomainError, Sequence> => {
    const trimmed = draft.name.trim();
    if (trimmed.length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (trimmed.length > 128) return Result.err({ code: 'TITLE_TOO_LONG', max: 128 });

    const now = Date.now();
    const items: SequenceItem[] = (draft.items ?? []).map((it, idx) => ({
      ...it,
      order: idx,
    }));

    return Result.ok({
      id: SequenceId.of(crypto.randomUUID()),
      name: trimmed,
      items,
      separator: draft.separator ?? { _type: 'newline', count: 1 },
      tags: [],
      favorite: false,
      createdAt: now,
      updatedAt: now,
      lastRenderedAt: null,
    });
  },

  renderSequence: (
    resolvedItems: readonly { content: string; item: SequenceItem }[],
    separator: SequenceSeparator
  ): string => {
    const parts = resolvedItems
      .filter(x => x.item.enabled)
      .map(({ content, item }, idx) => {
        let prefix = item.prefixOverride ?? '';
        let suffix = item.suffixOverride ?? '';

        if (!item.prefixOverride) {
          if (separator._type === 'numbered_list') {
            prefix = `${idx + 1}. `;
          } else if (separator._type === 'markdown_section') {
            prefix = `## Abschnitt ${idx + 1}\n\n`;
          }
        }

        return `${prefix}${content}${suffix}`;
      });

    let joiner = '';
    if (separator._type === 'newline') {
      joiner = '\n'.repeat(separator.count || 1);
    } else if (separator._type === 'custom') {
      joiner = separator.text;
    } else if (separator._type === 'numbered_list' || separator._type === 'markdown_section') {
      joiner = '\n\n';
    }

    return parts.join(joiner);
  },
} as const;
