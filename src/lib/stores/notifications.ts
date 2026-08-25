import { writable } from 'svelte/store';
import { DomainError } from '../domain/errors';
import { Option } from '../domain/adts';
import type { UnixMs } from '../domain/adts';

export type NotificationSeverity = 'success' | 'info' | 'warning' | 'error';

export interface NotificationAction {
  readonly label:   string;
  readonly handler: string;
}

export interface AppNotification {
  readonly id:          string;
  readonly severity:    NotificationSeverity;
  readonly title:       string;
  readonly message:     Option<string>;
  readonly duration:    number;
  readonly action:      Option<NotificationAction>;
  readonly createdAt:   UnixMs;
}

const generateId = () => crypto.randomUUID();
const now = () => Date.now() as UnixMs;

export const Notifications = {
  snippetSaved:      (title: string): AppNotification => ({ id: generateId(), severity: 'success', title: 'Gespeichert', message: Option.some(`"${title}" wurde gespeichert`),    duration: 1500, action: Option.none(), createdAt: now() }),
  snippetCopied:     ():              AppNotification => ({ id: generateId(), severity: 'success', title: 'Kopiert',     message: Option.none(),                                      duration: 1200, action: Option.none(), createdAt: now() }),
  transformComplete: (ms: number):    AppNotification => ({ id: generateId(), severity: 'success', title: 'Transformation abgeschlossen', message: Option.some(`In ${ms}ms`),        duration: 2000, action: Option.none(), createdAt: now() }),
  transformError:    (e: DomainError):AppNotification => ({ id: generateId(), severity: 'error',   title: 'Fehler',      message: Option.some(DomainError.describe(e)),               duration: 5000, action: Option.none(), createdAt: now() }),
  undoAvailable:     (desc: string):  AppNotification => ({ id: generateId(), severity: 'info',    title: 'Rückgängig möglich', message: Option.some(desc),                          duration: 3000, action: Option.some({ label: 'Rückgängig', handler: 'undo' }), createdAt: now() }),
} as const;

const _rawNotificationsStore = writable<AppNotification[]>([]);

export function pushNotification(notification: AppNotification) {
    _rawNotificationsStore.update(n => [...n, notification]);
    if (notification.duration > 0) {
        setTimeout(() => {
            dismissNotification(notification.id);
        }, notification.duration);
    }
}

export function dismissNotification(id: string) {
    _rawNotificationsStore.update(n => n.filter(x => x.id !== id));
}

export const notificationsStore = {
    subscribe: _rawNotificationsStore.subscribe,
    set: _rawNotificationsStore.set,
    update: _rawNotificationsStore.update,
    push: pushNotification
};
