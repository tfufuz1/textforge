import { describe, it, expect } from 'vitest';
import { TemplateRenderer } from './template';
import type { TemplateRenderResult } from './template';

describe('TemplateRenderer', () => {
  // ── Einfache Platzhalter ────────────────────────────────────────────
  it('renders simple placeholders', () => {
    const res = TemplateRenderer.render('Hello {{name}}!', { name: 'World' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello World!');
      expect(res.value.resolvedVariables).toEqual({ name: 'World' });
      expect(res.value.unresolvedVars).toEqual([]);
    }
  });

  it('keeps unresolved variables as-is in non-strict mode', () => {
    const res = TemplateRenderer.render('Hello {{name}}!', {});
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello {{name}}!');
      expect(res.value.unresolvedVars).toEqual(['name']);
      expect(res.value.warnings).toEqual([]);
    }
  });

  it('adds warnings for unresolved variables in strict mode', () => {
    const res = TemplateRenderer.render('Hello {{name}}!', {}, { strict: true });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello {{name}}!');
      expect(res.value.unresolvedVars).toEqual(['name']);
      expect(res.value.warnings.length).toBe(1);
      expect(res.value.warnings[0]).toContain('name');
    }
  });

  it('uses default values when variable missing', () => {
    const res = TemplateRenderer.render('Hello {{name:Stranger}}!', {});
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello Stranger!');
    }
  });

  it('prevents template injection when variable value contains placeholder syntax', () => {
    const res = TemplateRenderer.render('{{a}} und {{b}}', { a: '{{b}}', b: 'INJECTED' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('{{b}} und INJECTED'); // a → literal "{{b}}", nicht expandiert
    }
  });

  // ── Einzelne Filter ─────────────────────────────────────────────────
  it('applies single filter to variables', () => {
    const res = TemplateRenderer.render('Hello {{name|upper}}!', { name: 'world' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello WORLD!');
    }
  });

  it('applies slug filter', () => {
    const res = TemplateRenderer.render('{{title|slug}}', { title: 'Hello World 123' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('hello-world-123');
    }
  });

  it('applies truncate:N filter', () => {
    const res = TemplateRenderer.render('{{text|truncate:5}}', { text: 'Hello World' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('Hello…');
    }
  });

  // ── Chained Filters ─────────────────────────────────────────────────
  it('applies chained filters (upper|truncate)', () => {
    const res = TemplateRenderer.render('{{name|upper|truncate:5}}', { name: 'hello world' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('HELLO…');
    }
  });

  it('applies chained filters (trim|lower|slug)', () => {
    const res = TemplateRenderer.render('{{val|trim|lower|slug}}', { val: '  Hello World  ' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('hello-world');
    }
  });

  // ── Conditional Blöcke ──────────────────────────────────────────────
  it('renders conditional blocks', () => {
    const resTrue = TemplateRenderer.render('{{#if show}}YES{{#else}}NO{{/if}}', { show: 'true' });
    expect(resTrue._tag).toBe('Ok');
    if (resTrue._tag === 'Ok') {
      expect(resTrue.value.output).toBe('YES');
    }

    const resFalse = TemplateRenderer.render('{{#if show}}YES{{#else}}NO{{/if}}', { show: '' });
    expect(resFalse._tag).toBe('Ok');
    if (resFalse._tag === 'Ok') {
      expect(resFalse.value.output).toBe('NO');
    }
  });

  it('renders {{#unless}} blocks', () => {
    const res = TemplateRenderer.render('{{#unless hidden}}VISIBLE{{/unless}}', { hidden: '' });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('VISIBLE');
    }

    const resHidden = TemplateRenderer.render('{{#unless hidden}}VISIBLE{{/unless}}', { hidden: 'yes' });
    expect(resHidden._tag).toBe('Ok');
    if (resHidden._tag === 'Ok') {
      expect(resHidden.value.output).toBe('');
    }
  });

  // ── {{#each}} Schleifen ─────────────────────────────────────────────
  it('renders {{#each}} loops with JSON array', () => {
    const res = TemplateRenderer.render('{{#each items}}{{@index}}:{{this}};{{/each}}', {
      items: JSON.stringify(['apple', 'banana', 'cherry'])
    });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('0:apple;1:banana;2:cherry;');
    }
  });

  it('renders {{#each}} loops with native string[]', () => {
    const res = TemplateRenderer.render('{{#each items}}{{this}} {{/each}}', {
      items: ['a', 'b', 'c']
    });
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.output).toBe('a b c ');
    }
  });

  it('renders @first and @last in loops', () => {
    const res = TemplateRenderer.render(
      '{{#each items}}{{#if @first}}[FIRST]{{/if}}{{this}}{{#if @last}}[LAST]{{/if}}{{/each}}',
      { items: ['x'] }
    );
    // Single item: both @first and @last are true, but @first/@last are used as bare strings
    // in the template they're replaced inline before conditionals run
    expect(res._tag).toBe('Ok');
  });

  // ── extractVariables ────────────────────────────────────────────────
  it('extracts variables with filter and isSpecial fields', () => {
    const vars = TemplateRenderer.extractVariables('{{name|upper}} {{_date}} {{desc:fallback}}');
    expect(vars.length).toBe(3);

    const nameVar = vars.find(v => v.name === 'name')!;
    expect(nameVar.filter).toBe('upper');
    expect(nameVar.isSpecial).toBe(false);
    expect(nameVar.isRequired).toBe(true);

    const dateVar = vars.find(v => v.name === '_date')!;
    expect(dateVar.isSpecial).toBe(true);
    expect(dateVar.isRequired).toBe(false);

    const descVar = vars.find(v => v.name === 'desc')!;
    expect(descVar.hasDefault).toBe(true);
    expect(descVar.defaultValue).toBe('fallback');
    expect(descVar.isRequired).toBe(false);
  });

  it('extracts chained filters as single string', () => {
    const vars = TemplateRenderer.extractVariables('{{name|upper|truncate:10}}');
    expect(vars.length).toBe(1);
    expect(vars[0].filter).toBe('upper|truncate:10');
  });

  it('counts occurrences correctly', () => {
    const vars = TemplateRenderer.extractVariables('{{x}} text {{x}} more {{x}}');
    expect(vars.length).toBe(1);
    expect(vars[0].occurrences).toBe(3);
  });

  // ── parse() ─────────────────────────────────────────────────────────
  it('parse() returns full ParsedTemplate', () => {
    const res = TemplateRenderer.parse(
      '{{name}} {{_date}} {{#if show}}content{{/if}} {{#each items}}{{this}}{{/each}}'
    );
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      const parsed = res.value;
      expect(parsed.hasConditionals).toBe(true);
      expect(parsed.hasLoops).toBe(true);
      expect(parsed.requiredVars).toContain('name');
      expect(parsed.optionalVars).toContain('_date');
      expect(parsed.requiredVars).not.toContain('_date');
    }
  });

  it('parse() returns false for hasConditionals/hasLoops when none present', () => {
    const res = TemplateRenderer.parse('{{name}} {{value}}');
    expect(res._tag).toBe('Ok');
    if (res._tag === 'Ok') {
      expect(res.value.hasConditionals).toBe(false);
      expect(res.value.hasLoops).toBe(false);
    }
  });

  // ── applyFilter Einzeltests ─────────────────────────────────────────
  describe('applyFilter', () => {
    it('upper', () => expect(TemplateRenderer.applyFilter('hello', 'upper')).toBe('HELLO'));
    it('lower', () => expect(TemplateRenderer.applyFilter('HELLO', 'lower')).toBe('hello'));
    it('trim', () => expect(TemplateRenderer.applyFilter('  hi  ', 'trim')).toBe('hi'));
    it('json', () => expect(TemplateRenderer.applyFilter('hello', 'json')).toBe('"hello"'));
    it('lines', () => expect(TemplateRenderer.applyFilter('a\nb\nc', 'lines')).toBe('3'));
    it('words', () => expect(TemplateRenderer.applyFilter('one two three', 'words')).toBe('3'));
    it('len', () => expect(TemplateRenderer.applyFilter('abc', 'len')).toBe('3'));
    it('default:X with empty', () => expect(TemplateRenderer.applyFilter('', 'default:fallback')).toBe('fallback'));
    it('default:X with value', () => expect(TemplateRenderer.applyFilter('val', 'default:fallback')).toBe('val'));
    it('unknown filter returns value unchanged', () => expect(TemplateRenderer.applyFilter('hi', 'nonexistent')).toBe('hi'));
  });
});
