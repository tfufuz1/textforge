import { Option, ClipEntryId } from './adts';
import type { SnippetId, UnixMs, ByteSize, TagName } from './adts';
import { detectContentType } from './snippet';
import type { ContentType, SnippetLocation } from './snippet';

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
  readonly sourceMimeTypes?: ReadonlyArray<string>;
  readonly mimeData?:    Option<string>;

  readonly promotedToSnippetId: Option<SnippetId>;
}

export const ClipboardEntry = {
  computeHash: (content: string): string => {
    let h0 = 0x6a09e667, h1 = 0xbb67ae85, h2 = 0x3c6ef372, h3 = 0xa54ff53a;
    let h4 = 0x510e527f, h5 = 0x9b05688c, h6 = 0x1f83d9ab, h7 = 0x5be0cd19;
    const K = [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    ];
    const bytes = new TextEncoder().encode(content);
    const l = bytes.length;
    const bitLen = l * 8;
    const blocksCount = Math.floor((l + 9 + 63) / 64);
    const padded = new Uint8Array(blocksCount * 64);
    padded.set(bytes);
    padded[l] = 0x80;

    const view = new DataView(padded.buffer);
    const highBits = Math.floor(bitLen / 0x100000000);
    const lowBits = bitLen % 0x100000000;
    view.setUint32(padded.length - 8, highBits, false);
    view.setUint32(padded.length - 4, lowBits, false);

    const w = new Uint32Array(64);
    for (let chunk = 0; chunk < padded.length; chunk += 64) {
      for (let i = 0; i < 16; i++) {
        w[i] = view.getUint32(chunk + i * 4, false);
      }
      for (let i = 16; i < 64; i++) {
        const s0 = ((w[i - 15] >>> 7) | (w[i - 15] << 25)) ^ ((w[i - 15] >>> 18) | (w[i - 15] << 14)) ^ (w[i - 15] >>> 3);
        const s1 = ((w[i - 2] >>> 17) | (w[i - 2] << 15)) ^ ((w[i - 2] >>> 19) | (w[i - 2] << 13)) ^ (w[i - 2] >>> 10);
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) | 0;
      }
      let a = h0, b = h1, c = h2, d = h3, e = h4, f = h5, g = h6, h = h7;
      for (let i = 0; i < 64; i++) {
        const S1 = ((e >>> 6) | (e << 26)) ^ ((e >>> 11) | (e << 21)) ^ ((e >>> 25) | (e << 7));
        const ch = (e & f) ^ (~e & g);
        const temp1 = (h + S1 + ch + K[i] + w[i]) | 0;
        const S0 = ((a >>> 2) | (a << 30)) ^ ((a >>> 13) | (a << 19)) ^ ((a >>> 22) | (a << 10));
        const maj = (a & b) ^ (a & c) ^ (b & c);
        const temp2 = (S0 + maj) | 0;

        h = g;
        g = f;
        f = e;
        e = (d + temp1) | 0;
        d = c;
        c = b;
        b = a;
        a = (temp1 + temp2) | 0;
      }
      h0 = (h0 + a) | 0;
      h1 = (h1 + b) | 0;
      h2 = (h2 + c) | 0;
      h3 = (h3 + d) | 0;
      h4 = (h4 + e) | 0;
      h5 = (h5 + f) | 0;
      h6 = (h6 + g) | 0;
      h7 = (h7 + h) | 0;
    }
    const toHex = (n: number) => (n >>> 0).toString(16).padStart(8, '0');
    return toHex(h0) + toHex(h1) + toHex(h2) + toHex(h3) + toHex(h4) + toHex(h5) + toHex(h6) + toHex(h7);
  },

  create: (content: string, sourceApp: Option<string>): Option<ClipboardEntry> => {
    if (content.trim().length === 0) return Option.none();
    const bytes = new TextEncoder().encode(content);
    const now = Date.now() as UnixMs;
    const contentHash = ClipboardEntry.computeHash(content);
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
  },

  toSnippetDraft: (entry: ClipboardEntry): { title: string; content: string; location: SnippetLocation } => ({
    title:    entry.content.slice(0, 60).trim() || 'Clipboard-Import',
    content:  entry.content,
    location: { _type: 'inbox' },
  }),
} as const;
