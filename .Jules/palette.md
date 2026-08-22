## 2025-08-22 - Icon-only buttons accessibility pattern
**Learning:** Icon-only buttons (such as undo/redo or toolbar action buttons) without `aria-label` are inaccessible to screen reader users, even when `title` is present on some browsers/renderers.
**Action:** Always provide explicit `aria-label` attributes on icon-only interactive controls.
