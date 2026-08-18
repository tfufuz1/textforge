export interface TextStats {
  readonly charCount: number;
  readonly charCountNoSpaces: number;
  readonly wordCount: number;
  readonly lineCount: number;
  readonly nonEmptyLineCount: number;
  readonly paragraphCount: number;
  readonly estimatedReadingTimeSeconds: number;
  readonly estimatedTokens: number;
}

export function computeTextStats(text: string): TextStats {
  const charCount = text.length;
  const charCountNoSpaces = text.replace(/\s/g, '').length;
  const words = text.trim().split(/\s+/).filter(Boolean);
  const wordCount = text.trim() ? words.length : 0;
  const lines = text.split(/\r?\n/);
  const lineCount = lines.length;
  const nonEmptyLineCount = lines.filter((l) => l.trim().length > 0).length;
  const paragraphs = text.split(/\n\s*\n/).filter((p) => p.trim().length > 0);
  const paragraphCount = paragraphs.length;

  const estimatedReadingTimeSeconds = Math.ceil((wordCount / 200) * 60);
  const estimatedTokens = Math.ceil(wordCount * 1.3);

  return {
    charCount,
    charCountNoSpaces,
    wordCount,
    lineCount,
    nonEmptyLineCount,
    paragraphCount,
    estimatedReadingTimeSeconds,
    estimatedTokens,
  };
}
