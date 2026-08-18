import { Option, Result } from './adts';
import type { SnippetId, UnixMs } from './adts';
import type { Snippet, SnippetLocation } from './snippet';
import { DomainError } from './errors';

export type UndoAction =
  | { readonly _type: 'snippet_update';     readonly before: Snippet;   readonly after: Snippet }
  | { readonly _type: 'snippet_create';     readonly created: Snippet }
  | { readonly _type: 'snippet_delete';     readonly deleted: Snippet }
  | { readonly _type: 'snippet_move';       readonly id: SnippetId; readonly from: SnippetLocation; readonly to: SnippetLocation }
  | { readonly _type: 'script_update';      readonly before: any;       readonly after: any }
  | { readonly _type: 'script_create';      readonly created: any }
  | { readonly _type: 'script_delete';      readonly deleted: any }
  | { readonly _type: 'pipeline_update';    readonly before: any;       readonly after: any }
  | { readonly _type: 'transform_apply';    readonly snippetId: SnippetId;
      readonly originalContent: string;   readonly transformedContent: string;
      readonly pipelineId: Option<string>; readonly scriptId: Option<string> }
  | { readonly _type: 'bulk_operation';     readonly operations: ReadonlyArray<UndoAction> }
  | { readonly _type: 'folder_create';      readonly created: any }
  | { readonly _type: 'folder_rename';      readonly id: string; readonly from: string; readonly to: string }
  | { readonly _type: 'folder_delete';      readonly deleted: any; readonly movedSnippets: ReadonlyArray<SnippetId> };

export interface UndoEntry {
  readonly action:      UndoAction;
  readonly performedAt: UnixMs;
  readonly description: string;
}

export interface UndoStack {
  readonly undoable: ReadonlyArray<UndoEntry>;
  readonly redoable: ReadonlyArray<UndoEntry>;
  readonly maxSize:  number;
}

export const UndoStack = {
  empty:   (maxSize = 50): UndoStack => ({ undoable: [], redoable: [], maxSize }),

  push: (stack: UndoStack, entry: UndoEntry): UndoStack => ({
    undoable: [entry, ...stack.undoable].slice(0, stack.maxSize),
    redoable: [],
    maxSize:  stack.maxSize,
  }),

  undo: (stack: UndoStack): Result<DomainError, { entry: UndoEntry; newStack: UndoStack }> => {
    if (stack.undoable.length === 0) return Result.err({ code: 'UNDO_STACK_EMPTY' });
    const [entry, ...rest] = stack.undoable;
    return Result.ok({
      entry,
      newStack: { undoable: rest, redoable: [entry, ...stack.redoable], maxSize: stack.maxSize },
    });
  },

  redo: (stack: UndoStack): Result<DomainError, { entry: UndoEntry; newStack: UndoStack }> => {
    if (stack.redoable.length === 0) return Result.err({ code: 'REDO_STACK_EMPTY' });
    const [entry, ...rest] = stack.redoable;
    return Result.ok({
      entry,
      newStack: { undoable: [entry, ...stack.undoable], redoable: rest, maxSize: stack.maxSize },
    });
  },

  canUndo: (stack: UndoStack): boolean => stack.undoable.length > 0,
  canRedo: (stack: UndoStack): boolean => stack.redoable.length > 0,
} as const;