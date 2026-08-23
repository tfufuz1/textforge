## 2025-08-22 - Icon-only buttons accessibility pattern
**Learning:** Icon-only buttons (such as undo/redo or toolbar action buttons) without `aria-label` are inaccessible to screen reader users, even when `title` is present on some browsers/renderers.
**Action:** Always provide explicit `aria-label` attributes on icon-only interactive controls.

## 2025-08-23 - Notification region accessibility pattern
**Learning:** Toast containers require `role="region"`, `aria-label`, and `aria-live="polite"` so screen readers announce toasts dynamically without interrupting current user flow. Decorative severity icons should be marked `aria-hidden="true"`.
**Action:** Always include live region attributes and explicit close button ARIA labels when creating or updating notification overlay components.
