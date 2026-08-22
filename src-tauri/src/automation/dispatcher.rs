use std::sync::atomic::{AtomicBool, Ordering};

/// INVARIANT-AR2: Feedback-Loop-Guard
/// Verhindert, dass durch eigene write_to_clipboard Aktionen erneut 'on_clipboard_change' getriggert wird.
pub static IS_INTERNAL_CLIPBOARD_WRITE: AtomicBool = AtomicBool::new(false);

pub struct AutomationDispatcher;

impl AutomationDispatcher {
    pub fn set_internal_write_guard(is_writing: bool) {
        IS_INTERNAL_CLIPBOARD_WRITE.store(is_writing, Ordering::SeqCst);
    }

    pub fn should_suppress_clipboard_event() -> bool {
        IS_INTERNAL_CLIPBOARD_WRITE.load(Ordering::SeqCst)
    }
}
