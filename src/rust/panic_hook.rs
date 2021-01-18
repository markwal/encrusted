use std::panic;
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn js_error(ptr: *const c_char);
}

fn emit_js_error(buf: &str) {
    if let Ok(cstring) = CString::new(buf) {
        unsafe {
            js_error(cstring.as_ptr());
        }
    }
    // we're panicking already, so we'll just eat it since sending it to stderr
    // doesn't go anywhere either (or we wouldn't need the hook)
}

/// A panic hook for use with
/// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html)
/// that logs panics into
/// [`console.error`](https://developer.mozilla.org/en-US/docs/Web/API/Console/error).
///
/// On non-wasm targets, prints the panic to `stderr`.
pub fn hook(info: &panic::PanicInfo) {
    let msg = info.to_string();

    emit_js_error(&msg);
}

/// Set the `console.error` pnaic hook the first time this is called.  Calling
/// it again does nothing
#[inline]
pub fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook));
    });
}
