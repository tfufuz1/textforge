use textforge::automation::dispatcher::AutomationDispatcher;
use textforge::commands::clipboard::ClipboardWriteGuard;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn test_clipboard_write_guard_acquire_and_drop() {
    // Before acquire, guard should be false
    AutomationDispatcher::set_internal_write_guard(false);
    assert!(!AutomationDispatcher::should_suppress_clipboard_event());

    {
        let _guard = ClipboardWriteGuard::acquire();
        assert!(AutomationDispatcher::should_suppress_clipboard_event());
    }

    // After scope exit, should_suppress_clipboard_event() must return false
    assert!(!AutomationDispatcher::should_suppress_clipboard_event());
}

#[test]
fn test_clipboard_write_guard_resets_on_panic() {
    AutomationDispatcher::set_internal_write_guard(false);
    assert!(!AutomationDispatcher::should_suppress_clipboard_event());

    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = ClipboardWriteGuard::acquire();
        assert!(AutomationDispatcher::should_suppress_clipboard_event());
        panic!("Simulated write_to_clipboard panic!");
    }));

    assert!(panic_result.is_err());

    // Guard must be reset to false after panic unwinds
    assert!(!AutomationDispatcher::should_suppress_clipboard_event());
}
