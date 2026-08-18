import { describe, it, expect } from 'vitest';
import { UndoStack } from './undo';
import type { UndoEntry } from './undo';

describe('UndoStack Domain Model', () => {
  const dummyEntry = (desc: string): UndoEntry => ({
    description: desc,
    performedAt: Date.now(),
    action: {
      _type: 'snippet_create',
      created: { id: '1', title: 'test', content: 'test content' } as any
    }
  });

  it('initializes as empty stack', () => {
    const stack = UndoStack.empty(10);
    expect(stack.maxSize).toBe(10);
    expect(UndoStack.canUndo(stack)).toBe(false);
    expect(UndoStack.canRedo(stack)).toBe(false);
  });

  it('pushes entries onto the stack and clears redo history', () => {
    let stack = UndoStack.empty(3);
    stack = UndoStack.push(stack, dummyEntry('action 1'));
    stack = UndoStack.push(stack, dummyEntry('action 2'));

    expect(stack.undoable.length).toBe(2);
    expect(stack.redoable.length).toBe(0);
    expect(stack.undoable[0].description).toBe('action 2');
    expect(stack.undoable[1].description).toBe('action 1');
  });

  it('enforces maximum stack size', () => {
    let stack = UndoStack.empty(2);
    stack = UndoStack.push(stack, dummyEntry('action 1'));
    stack = UndoStack.push(stack, dummyEntry('action 2'));
    stack = UndoStack.push(stack, dummyEntry('action 3'));

    expect(stack.undoable.length).toBe(2);
    expect(stack.undoable[0].description).toBe('action 3');
    expect(stack.undoable[1].description).toBe('action 2');
  });

  it('performs undo and redo transitions correctly', () => {
    let stack = UndoStack.empty(5);
    stack = UndoStack.push(stack, dummyEntry('action 1'));
    stack = UndoStack.push(stack, dummyEntry('action 2'));

    // Undo 1
    const res1 = UndoStack.undo(stack);
    expect(res1._tag).toBe('Ok');
    if (res1._tag === 'Ok') {
      expect(res1.value.entry.description).toBe('action 2');
      stack = res1.value.newStack;
    }
    expect(stack.undoable.length).toBe(1);
    expect(stack.redoable.length).toBe(1);
    expect(stack.undoable[0].description).toBe('action 1');
    expect(stack.redoable[0].description).toBe('action 2');

    // Redo 1
    const res2 = UndoStack.redo(stack);
    expect(res2._tag).toBe('Ok');
    if (res2._tag === 'Ok') {
      expect(res2.value.entry.description).toBe('action 2');
      stack = res2.value.newStack;
    }
    expect(stack.undoable.length).toBe(2);
    expect(stack.redoable.length).toBe(0);
  });
});
