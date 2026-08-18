import { Option, ClipEntryId } from './adts';
import type { SnippetId, UnixMs, ByteSize, TagName } from './adts';
import { detectContentType } from './snippet';
import type { ContentType } from './snippet';

export interface ClipboardEntry {
  readonly id:           ClipEntryId;
  readonly content:      string;
  readonly contentHash:  string;
  readonly contentType:  ContentType;
  readonly sourceApp:    Option<string>;
  readonly capturedAt:   UnixMs;

  readonly sizeBytes:    ByteSize;
  readonly lineCount:    number;
  readonly wordCount:    number;
  readonly isPinned:     boolean;
  readonly tags:         ReadonlyArray<TagName>;

  readonly promotedToSnippetId: Option<SnippetId>;
}

export const ClipboardEntry = {
  create: (content: string, sourceApp: Option<string>): Option<ClipboardEntry> => {
    if (content.trim().length === 0) return Option.none();
    const bytes = new TextEncoder().encode(content);
    const now = Date.now() as UnixMs;
    // contentHash should be generated but for now we just create a placeholder
    const contentHash = 'hash_' + now; // Placeholder
    return Option.some({
      id:           ClipEntryId.of(crypto.randomUUID()),
      content,
      contentHash,
      contentType:  detectContentType(content),
      sourceApp,
      capturedAt:   now,
      sizeBytes:    bytes.length as ByteSize,
      lineCount:    content.split('\n').length,
      wordCount:    content.trim() === '' ? 0 : content.trim().split(/\s+/).length,
      isPinned:     false,
      tags:         [],
      promotedToSnippetId: Option.none(),
    });
  }
} as const;
