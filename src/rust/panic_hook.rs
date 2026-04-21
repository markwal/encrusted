use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = __encrusted_js_error)]
    fn js_error(message: &str);
}

fn emit_js_error(buf: &str) {
    js_error(buf);
}

/// A panic hook for use with
/// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html)
/// that logs panics into
/// [`console.error`](https://developer.mozilla.org/en-US/docs/Web/API/Console/error).
///
/// On non-wasm targets, prints the panic to `stderr`.
pub fn hook(info: &panic::PanicHookInfo<'_>) {
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
