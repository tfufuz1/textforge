export type NotificationSeverity = 'info' | 'success' | 'warning' | 'error';

export interface AppNotification {
  readonly id: string;
  readonly message: string;
  readonly severity: NotificationSeverity;
  readonly timeoutMs?: number;
  readonly createdAt: number;
}

export const Notifications = {
  info: (message: string, timeoutMs = 3000): AppNotification => ({
    id: Math.random().toString(36).substring(2, 9),
    message,
    severity: 'info',
    timeoutMs,
    createdAt: Date.now(),
  }),
  success: (message: string, timeoutMs = 3000): AppNotification => ({
    id: Math.random().toString(36).substring(2, 9),
    message,
    severity: 'success',
    timeoutMs,
    createdAt: Date.now(),
  }),
  warning: (message: string, timeoutMs = 4000): AppNotification => ({
    id: Math.random().toString(36).substring(2, 9),
    message,
    severity: 'warning',
    timeoutMs,
    createdAt: Date.now(),
  }),
  error: (message: string, timeoutMs = 5000): AppNotification => ({
    id: Math.random().toString(36).substring(2, 9),
    message,
    severity: 'error',
    timeoutMs,
    createdAt: Date.now(),
  }),
} as const;
