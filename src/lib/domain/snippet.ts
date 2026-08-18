import { Option, Result, SnippetId } from './adts';
import type { FolderId, TagName, UnixMs, ByteSize } from './adts';
import { DomainError } from './errors';

export type ContentType = 'plain_text' | 'markdown' | 'html' | 'xml' | 'javascript' | 'typescript' | 'python' | 'rust' | 'go' | 'java' | 'kotlin' | 'swift' | 'cpp' | 'c' | 'csharp' | 'php' | 'ruby' | 'bash' | 'powershell' | 'json' | 'yaml' | 'toml' | 'csv' | 'sql' | 'graphql' | 'css' | 'scss' | 'less' | 'url' | 'file_path' | 'regex' | 'template' | 'unknown';

export const isValidJSON = (s: string): boolean => {
  try { JSON.parse(s); return true; } catch { return false; }
};

export const detectContentType = (content: string): ContentType => {
  const t = content.trim();
  if (/^https?:\/\//.test(t)) return 'url';
  if (/^(\/[\w.-]+)+\/?$/.test(t) || /^[A-Za-z]:\\/.test(t)) return 'file_path';
  if ((t.startsWith('{') || t.startsWith('[')) && isValidJSON(t)) return 'json';
  if (/\{\{[\w.]+\}\}/.test(t)) return 'template';
  if (/^---\n/.test(t) || /#{1,6}\s/.test(t) || /\*\*|__|\[.+\]\(/.test(t)) return 'markdown';
  if (/^<[a-zA-Z][^>]*>/.test(t) && /<\/[a-zA-Z]+>/.test(t)) return 'html';
  if (/^<\?xml|^<[a-zA-Z]+:[a-zA-Z]+/.test(t)) return 'xml';
  if (/^(const|let|var|function|import|export|class|=>)\s/.test(t) && !t.includes('def ')) return 'javascript';
  if (/^(interface|type |enum |namespace|declare)\s/.test(t)) return 'typescript';
  if (/^(def |class |import |from .* import|async def|@\w+\n)/.test(t)) return 'python';
  if (/^(fn |use |mod |struct |impl |pub |enum |trait )/.test(t)) return 'rust';
  if (/^(package |import "|(func|type|var|const) \w)/.test(t)) return 'go';
  if (/^(public |private |class |interface |import java)/.test(t)) return 'java';
  if (/^(fun |val |var |data class|object |companion)/.test(t)) return 'kotlin';
  if (/^(import (Foundation|UIKit|SwiftUI)|func |var |let |class )/.test(t)) return 'swift';
  if (/^(#include|int main|void |printf|struct |typedef)/.test(t)) return 'c';
  if (/^(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\s/i.test(t)) return 'sql';
  if (/^(query|mutation|subscription|type \w+ \{)/.test(t)) return 'graphql';
  if (/^(#!\/bin\/(bash|sh|zsh)|echo |grep |sed |awk )/.test(t)) return 'bash';
  if (/^(---|\w+:\s*\n  \w+:)/.test(t)) return 'yaml';
  if (/^\[[\w.-]+\]\n\w+ *= /.test(t)) return 'toml';
  if (/^(\.|#)[a-zA-Z][\w-]*\s*\{/.test(t)) return 'css';
  return 'plain_text';
};

export type SnippetLocation =
  | { readonly _type: 'inbox'   }
  | { readonly _type: 'archive' }
  | { readonly _type: 'trash';  readonly deletedAt: UnixMs }
  | {
      readonly _type:    'folder';
      readonly folderId: FolderId;
      readonly path:     ReadonlyArray<string>;
    };

export interface Snippet {
  readonly id:          SnippetId;
  readonly title:       string;
  readonly content:     string;
  readonly tags:        ReadonlyArray<TagName>;
  readonly location:    SnippetLocation;
  readonly contentType: ContentType;
  readonly createdAt:   UnixMs;
  readonly updatedAt:   UnixMs;
  readonly usageCount:  number;
  readonly isPinned:    boolean;
  readonly sourceApp:   Option<string>;
  readonly isTemplate:  boolean;
  readonly color:       Option<string>;
  readonly favorite:    boolean;
}

export type SnippetPatch = Partial<Pick<Snippet,
  'title' | 'content' | 'tags' | 'location' | 'isPinned' | 'color' | 'favorite'
>>;

export const Snippet = {
  create: (draft: {
    title:    string;
    content:  string;
    location: SnippetLocation;
    sourceApp?: string;
  }): Result<DomainError, Snippet> => {
    if (draft.title.trim().length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (draft.title.length > 128)        return Result.err({ code: 'TITLE_TOO_LONG', max: 128 });
    if (draft.content.length > 10 * 1024 * 1024)
      return Result.err({ code: 'CONTENT_TOO_LARGE', maxBytes: 10 * 1024 * 1024 });

    const now = Date.now() as UnixMs;
    const content = draft.content;
    return Result.ok({
      id:          SnippetId.of(crypto.randomUUID()),
      title:       draft.title.trim(),
      content,
      tags:        [],
      location:    draft.location,
      contentType: detectContentType(content),
      isTemplate:  /\{\{[\w.]+\}\}/.test(content),
      createdAt:   now,
      updatedAt:   now,
      usageCount:  0,
      isPinned:    false,
      favorite:    false,
      sourceApp:   Option.fromNullable(draft.sourceApp),
      color:       Option.none(),
    });
  },

  update: (snippet: Snippet, patch: SnippetPatch): Result<DomainError, Snippet> => {
    const merged = { ...snippet, ...patch, updatedAt: Date.now() as UnixMs };
    if (patch.content !== undefined) {
      merged.contentType = detectContentType(merged.content);
      merged.isTemplate  = /\{\{[\w.]+\}\}/.test(merged.content);
    }
    return Snippet.validate(merged);
  },

  duplicate: (original: Snippet): Result<DomainError, Snippet> =>
    Snippet.create({
      title:    `${original.title} (Kopie)`,
      content:  original.content,
      location: original.location,
    }),

  validate: (s: Snippet): Result<DomainError, Snippet> => {
    if (!s.title || s.title.trim().length === 0) return Result.err({ code: 'EMPTY_TITLE' });
    if (s.title.length > 128)                    return Result.err({ code: 'TITLE_TOO_LONG', max: 128 });
    if (new Set(s.tags).size !== s.tags.length)  return Result.err({ code: 'DUPLICATE_TAGS' });
    if (s.tags.length > 20)                      return Result.err({ code: 'TOO_MANY_TAGS', max: 20 });
    if (s.color._tag === 'Some' && !/^#[0-9A-Fa-f]{6}$/.test(s.color.value))
      return Result.err({ code: 'INVALID_COLOR', value: s.color.value });
    return Result.ok(s);
  },

  sizeBytes:  (s: Snippet): ByteSize  => new TextEncoder().encode(s.content).length as ByteSize,
  wordCount:  (s: Snippet): number    => s.content.trim() === '' ? 0 : s.content.trim().split(/\s+/).length,
  lineCount:  (s: Snippet): number    => s.content.split('\n').length,
  charCount:  (s: Snippet): number    => s.content.length,
  charNoSpaceCount: (s: Snippet): number => s.content.replace(/\s/g, '').length,
} as const;
