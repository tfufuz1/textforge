import { Result } from './adts';
import type { DomainError } from './errors';

// § 6.2 — TemplateVariable
export interface TemplateVariable {
  readonly name:        string;
  readonly hasDefault:  boolean;
  readonly defaultValue: string;
  readonly filter:      string | null;   // z.B. 'upper', 'truncate:100', 'upper|truncate:10' (chained)
  readonly isSpecial:   boolean;         // Beginnt mit _ → automatisch gesetzt
  readonly isRequired:  boolean;         // Kein Default, kein Special
  readonly occurrences: number;
}

// § 6.2 — ParsedTemplate
export interface ParsedTemplate {
  readonly rawText:         string;
  readonly variables:       readonly TemplateVariable[];
  readonly requiredVars:    readonly string[];   // name, !hasDefault && !isSpecial
  readonly optionalVars:    readonly string[];   // hasDefault || isSpecial
  readonly hasConditionals: boolean;
  readonly hasLoops:        boolean;
}

// § 6.2 — TemplateContext (string | string[] für {{#each}} Arrays)
export type TemplateContext = Readonly<Record<string, string | string[]>>;

// § 6.2 — TemplateRenderResult
export interface TemplateRenderResult {
  readonly output:             string;
  readonly resolvedVariables:  Readonly<Record<string, string>>;
  readonly unresolvedVars:     readonly string[];
  readonly warnings:           readonly string[];
}

// ── Variable-Regex ────────────────────────────────────────────────────
// Captures: [1]=name  [2]=default (optional)  [3]=filters chain (optional, e.g. "|upper|truncate:10")
const VAR_REGEX = /\{\{\s*([a-zA-Z0-9_.-]+)(?::([^|{}]+))?((?:\|[a-zA-Z0-9_:]+)*)\s*\}\}/g;

// Block-Tag-Check: #if, /if, #else, #each, /each, @index, @first, @last, this, else
function isBlockTag(name: string): boolean {
  return name.startsWith('#') || name.startsWith('/') || name.startsWith('@')
    || name === 'this' || name === 'else';
}

export const TemplateRenderer = {
  /** Wendet einen einzelnen Filter-Operator auf einen Wert an (§ 6.1) */
  applyFilter: (val: string, filter: string): string => {
    if (filter.startsWith('truncate:')) {
      const n = parseInt(filter.slice('truncate:'.length), 10) || 100;
      return val.length > n ? val.slice(0, n) + '…' : val;
    }
    if (filter.startsWith('default:')) {
      const fallback = filter.slice('default:'.length);
      return val === '' ? fallback : val;
    }

    switch (filter) {
      case 'upper': return val.toUpperCase();
      case 'lower': return val.toLowerCase();
      case 'title': return val.replace(/\w\S*/g, (txt) => txt.charAt(0).toUpperCase() + txt.substring(1).toLowerCase());
      case 'trim': return val.trim();
      case 'slug': return val.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
      case 'snake': return val.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase().replace(/[\s-]+/g, '_');
      case 'camel': return val.toLowerCase().replace(/[-_\s]+(.)?/g, (_, c) => c ? c.toUpperCase() : '');
      case 'pascal': return val.replace(/[-_\s]+(.)?/g, (_, c) => c ? c.toUpperCase() : '').replace(/^(.)/, (c) => c.toUpperCase());
      case 'json': return JSON.stringify(val);
      case 'base64': {
        try { return btoa(unescape(encodeURIComponent(val))); } catch { return val; }
      }
      case 'url': return encodeURIComponent(val);
      case 'reverse': return [...val].reverse().join('');
      case 'first': return val.split(/\r?\n/)[0] || '';
      case 'last': {
        const lines = val.split(/\r?\n/);
        return lines[lines.length - 1] || '';
      }
      case 'lines': return val.split(/\r?\n/).length.toString();
      case 'words': return (val.trim() === '' ? 0 : val.trim().split(/\s+/).length).toString();
      case 'len': return val.length.toString();
      default: return val;
    }
  },

  /** Wendet eine Kette von Filtern an (z.B. "|upper|truncate:10" → ["upper", "truncate:10"]) */
  applyFilters: (val: string, filterChain: string): string => {
    if (!filterChain) return val;
    // filterChain kommt als "|upper|truncate:10" oder "upper|truncate:10"
    const filters = filterChain
      .split('|')
      .map(f => f.trim())
      .filter(f => f.length > 0);
    return filters.reduce((acc, f) => TemplateRenderer.applyFilter(acc, f), val);
  },

  /** § 6.3 — Extrahiert alle Variablen aus einem Template */
  extractVariables: (templateText: string): TemplateVariable[] => {
    const vars = new Map<string, TemplateVariable>();
    const regex = new RegExp(VAR_REGEX.source, VAR_REGEX.flags);
    let match;
    while ((match = regex.exec(templateText)) !== null) {
      const name = match[1];

      if (isBlockTag(name)) continue;

      const hasDefault = match[2] !== undefined;
      const defaultVal = match[2] || '';
      // match[3] is the chained filter string, e.g. "|upper|truncate:10" or ""
      const filterRaw = match[3] || '';
      const filterStr = filterRaw.startsWith('|') ? filterRaw.slice(1) : filterRaw;
      const filter = filterStr || null;

      if (vars.has(name)) {
        const existing = vars.get(name)!;
        vars.set(name, {
          ...existing,
          occurrences: existing.occurrences + 1,
          hasDefault: existing.hasDefault || hasDefault,
          defaultValue: existing.hasDefault ? existing.defaultValue : defaultVal,
          isRequired: existing.isRequired && !hasDefault,
          // Keep first filter seen
          filter: existing.filter ?? filter,
        });
      } else {
        vars.set(name, {
          name,
          hasDefault,
          defaultValue: defaultVal,
          filter,
          isSpecial: name.startsWith('_'),
          isRequired: !hasDefault && !name.startsWith('_'),
          occurrences: 1,
        });
      }
    }

    // Scan for {{#each NAME}} loop variables
    const eachRegex = /\{\{#each\s+([a-zA-Z0-9_\-]+)\}\}/g;
    let eachMatch;
    while ((eachMatch = eachRegex.exec(templateText)) !== null) {
      const name = eachMatch[1];
      if (vars.has(name)) {
        const existing = vars.get(name)!;
        vars.set(name, {
          ...existing,
          occurrences: existing.occurrences + 1,
        });
      } else {
        vars.set(name, {
          name,
          hasDefault: false,
          defaultValue: '',
          filter: null,
          isSpecial: name.startsWith('_'),
          isRequired: !name.startsWith('_'),
          occurrences: 1,
        });
      }
    }

    return Array.from(vars.values());
  },

  /** § 6.3 — Parst ein Template vollständig */
  parse: (templateText: string): Result<DomainError, ParsedTemplate> => {
    try {
      const variables = TemplateRenderer.extractVariables(templateText);
      const requiredVars = variables
        .filter(v => v.isRequired)
        .map(v => v.name);
      const optionalVars = variables
        .filter(v => !v.isRequired)
        .map(v => v.name);

      return Result.ok({
        rawText: templateText,
        variables,
        requiredVars,
        optionalVars,
        hasConditionals: /\{\{#if\s/.test(templateText) || /\{\{#unless\s/.test(templateText),
        hasLoops: /\{\{#each\s/.test(templateText),
      });
    } catch (e: any) {
      return Result.err({
        code: 'TEMPLATE_PARSE_ERROR',
        details: e.message || String(e),
      });
    }
  },

  /** § 6.3 — Rendert ein Template. Gibt TemplateRenderResult zurück. */
  render: (
    templateText: string,
    context: TemplateContext,
    options: { strict: boolean } = { strict: false },
  ): Result<DomainError, TemplateRenderResult> => {
    try {
      const resolvedVariables: Record<string, string> = {};
      const unresolvedVars: string[] = [];
      const warnings: string[] = [];

      // Kontext normalisieren: string[] → string (für einfache Variablen)
      const stringContext: Record<string, string> = {};
      for (const [k, v] of Object.entries(context)) {
        stringContext[k] = Array.isArray(v) ? JSON.stringify(v) : v;
      }

      // Schritt 1: {{#each var}}...{{/each}} Schleifen
      let processed = templateText.replace(/\{\{#each\s+([a-zA-Z0-9_\-]+)\}\}(.*?)\{\{\/each\}\}/gs, (_match: string, varName: string, body: string) => {
        const rawVal = context[varName];
        let items: string[] = [];
        if (Array.isArray(rawVal)) {
          items = rawVal;
        } else if (typeof rawVal === 'string') {
          try {
            const parsed = JSON.parse(rawVal);
            items = Array.isArray(parsed) ? parsed.map(String) : [];
          } catch {
            items = rawVal ? rawVal.split(/\r?\n/) : [];
          }
        }
        if (items.length === 0) return '';
        return items.map((item, idx) => {
          return body
            .replace(/\{\{this\}\}/g, String(item))
            .replace(/\{\{@index\}\}/g, String(idx))
            .replace(/\{\{@first\}\}/g, String(idx === 0))
            .replace(/\{\{@last\}\}/g, String(idx === items.length - 1));
        }).join('');
      });

      // Schritt 2: {{#if var}}...{{#else}}...{{/if}} Conditionals
      processed = processed.replace(/\{\{#if\s+(\w+)\}\}(.*?)(?:\{\{#else\}\}(.*?))?\{\{\/if\}\}/gs, (_match: string, varName: string, thenBlock: string, elseBlock: string = '') => {
        const val = stringContext[varName];
        const isTruthy = val !== undefined && val !== '' && val !== 'false' && val !== '0';
        return isTruthy ? thenBlock : elseBlock;
      });

      // Schritt 3: {{#unless var}}...{{/unless}}
      processed = processed.replace(/\{\{#unless\s+(\w+)\}\}(.*?)\{\{\/unless\}\}/gs, (_match: string, varName: string, block: string) => {
        const val = stringContext[varName];
        const isTruthy = val !== undefined && val !== '' && val !== 'false' && val !== '0';
        return !isTruthy ? block : '';
      });

      // Schritt 4: Variablen ersetzen (mit chained filter support)
      // Single-pass substitution via String.prototype.replace callback:
      // Prevents template injection because substituted variable values containing {{...}}
      // are inserted directly without undergoing additional replacement passes.
      const renderRegex = new RegExp(VAR_REGEX.source, VAR_REGEX.flags);
      const output = processed.replace(renderRegex, (match: string, varName: string, defaultVal: string | undefined, filterChain: string | undefined) => {
        if (isBlockTag(varName)) return match;

        let val: string | undefined = undefined;
        if (Object.prototype.hasOwnProperty.call(stringContext, varName)) {
          val = stringContext[varName];
        } else if (defaultVal !== undefined) {
          val = defaultVal;
        }

        if (val === undefined) {
          if (!unresolvedVars.includes(varName)) {
            unresolvedVars.push(varName);
          }
          if (options.strict) {
            warnings.push(`Fehlende Variable: {{${varName}}}`);
          }
          return match;
        }

        // Chained filters anwenden
        if (filterChain) {
          val = TemplateRenderer.applyFilters(val, filterChain);
        }

        resolvedVariables[varName] = val;
        return val;
      });

      return Result.ok({ output, resolvedVariables, unresolvedVars, warnings });
    } catch (e: any) {
      if (e.message && e.message.startsWith('Missing variable: ')) {
        return Result.err({ code: 'MISSING_TEMPLATE_VAR', variable: e.message.split(': ')[1] });
      }
      return Result.err({
        code: 'TEMPLATE_PARSE_ERROR',
        details: e.message || String(e),
      });
    }
  },
} as const;
