import { describe, it, expect } from 'vitest';
import {
  createDefaultExportRequest,
  validateExportRequest,
  type ExportRequest
} from './import-export';

describe('Import / Export Domain Logic', () => {
  it('creates default export request properly', () => {
    const req = createDefaultExportRequest('/tmp/backup.tfbundle');
    expect(req.exportType).toBe('full');
    expect(req.format).toBe('tfbundle');
    expect(req.targetPath).toBe('/tmp/backup.tfbundle');
    expect(req.includeSnippets).toBe(true);
  });

  it('validates export request correctly', () => {
    const valid = createDefaultExportRequest('/tmp/export.json');
    expect(validateExportRequest(valid)).toBe(true);

    const emptyPath: ExportRequest = {
      ...valid,
      targetPath: ''
    };
    expect(validateExportRequest(emptyPath)).toBe(false);

    const invalidFormat = {
      ...valid,
      format: 'unsupported_format' as any
    };
    expect(validateExportRequest(invalidFormat)).toBe(false);
  });
});
