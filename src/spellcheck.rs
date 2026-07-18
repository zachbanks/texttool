//! Optional system spell-check integration for `clean`.
//!
//! The implementation is intentionally isolated because it relies on
//! macOS-specific Foundation/AppKit APIs. On non-macOS targets the helper
//! compiles but returns a clear runtime error if the feature is requested.

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use crate::casing::capitalize;
    use std::mem::transmute;

    type Id = *mut c_void;
    type Sel = *mut c_void;
    type NSInteger = isize;
    type NSUInteger = usize;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSRange {
        location: NSUInteger,
        length: NSUInteger,
    }

    const NS_UTF8_STRING_ENCODING: NSUInteger = 4;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    fn cstr(bytes: &'static [u8]) -> &'static CStr {
        // All call sites pass static, nul-terminated selector/class names.
        unsafe { CStr::from_bytes_with_nul_unchecked(bytes) }
    }

    unsafe fn selector(bytes: &'static [u8]) -> Sel {
        unsafe { sel_registerName(cstr(bytes).as_ptr()) }
    }

    unsafe fn msg_send0<R>(receiver: Id, sel: Sel) -> R {
        let f: extern "C" fn(Id, Sel) -> R = unsafe { transmute(objc_msgSend as *const ()) };
        f(receiver, sel)
    }

    unsafe fn msg_send1<R, A1>(receiver: Id, sel: Sel, a1: A1) -> R {
        let f: extern "C" fn(Id, Sel, A1) -> R = unsafe { transmute(objc_msgSend as *const ()) };
        f(receiver, sel, a1)
    }

    unsafe fn msg_send3<R, A1, A2, A3>(receiver: Id, sel: Sel, a1: A1, a2: A2, a3: A3) -> R {
        let f: extern "C" fn(Id, Sel, A1, A2, A3) -> R =
            unsafe { transmute(objc_msgSend as *const ()) };
        f(receiver, sel, a1, a2, a3)
    }

    unsafe fn msg_send4<R, A1, A2, A3, A4>(
        receiver: Id,
        sel: Sel,
        a1: A1,
        a2: A2,
        a3: A3,
        a4: A4,
    ) -> R {
        let f: extern "C" fn(Id, Sel, A1, A2, A3, A4) -> R =
            unsafe { transmute(objc_msgSend as *const ()) };
        f(receiver, sel, a1, a2, a3, a4)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn msg_send6<R, A1, A2, A3, A4, A5, A6>(
        receiver: Id,
        sel: Sel,
        a1: A1,
        a2: A2,
        a3: A3,
        a4: A4,
        a5: A5,
        a6: A6,
    ) -> R {
        let f: extern "C" fn(Id, Sel, A1, A2, A3, A4, A5, A6) -> R =
            unsafe { transmute(objc_msgSend as *const ()) };
        f(receiver, sel, a1, a2, a3, a4, a5, a6)
    }

    fn is_titlecase(word: &str) -> bool {
        let mut chars = word.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        first.is_uppercase() && chars.all(|c| !c.is_alphabetic() || c.is_lowercase())
    }

    fn normalize_suggestion(original: &str, suggestion: &str) -> String {
        let core_is_all_caps = original.chars().all(|c| c.is_uppercase());
        if core_is_all_caps {
            suggestion.to_uppercase()
        } else if is_titlecase(original) {
            capitalize(suggestion)
        } else {
            suggestion.to_lowercase()
        }
    }

    fn rust_string_from_nsstring(string: Id) -> Result<String, String> {
        unsafe {
            let utf8: *const c_char = msg_send0(string, selector(b"UTF8String\0"));
            if utf8.is_null() {
                return Err("spell checker returned an invalid UTF-8 string".to_string());
            }
            Ok(CStr::from_ptr(utf8).to_string_lossy().into_owned())
        }
    }

    /// Thin wrapper around `NSSpellChecker.sharedSpellChecker`.
    pub(crate) struct SpellChecker {
        checker: Id,
    }

    impl SpellChecker {
        pub(crate) fn new() -> Result<Self, String> {
            unsafe {
                let pool = objc_autoreleasePoolPush();
                let checker_class = objc_getClass(cstr(b"NSSpellChecker\0").as_ptr());
                if checker_class.is_null() {
                    objc_autoreleasePoolPop(pool);
                    return Err("system spell checker is unavailable".to_string());
                }
                let checker: Id = msg_send0(checker_class, selector(b"sharedSpellChecker\0"));
                objc_autoreleasePoolPop(pool);
                if checker.is_null() {
                    Err("failed to access the system spell checker".to_string())
                } else {
                    Ok(Self { checker })
                }
            }
        }

        fn nsstring_from_str(&self, word: &str) -> Result<Id, String> {
            unsafe {
                let ns_string_class = objc_getClass(cstr(b"NSString\0").as_ptr());
                if ns_string_class.is_null() {
                    return Err("NSString class is unavailable".to_string());
                }
                let alloc: Id = msg_send0(ns_string_class, selector(b"alloc\0"));
                if alloc.is_null() {
                    return Err("failed to allocate NSString".to_string());
                }
                let string: Id = msg_send3(
                    alloc,
                    selector(b"initWithBytes:length:encoding:\0"),
                    word.as_ptr() as *const c_void,
                    word.len(),
                    NS_UTF8_STRING_ENCODING,
                );
                if string.is_null() {
                    Err("failed to convert text to NSString".to_string())
                } else {
                    Ok(string)
                }
            }
        }

        fn should_spellcheck(word: &str) -> bool {
            let core = crate::casing::core(word);
            if core.is_empty()
                || !core.chars().all(|c| c.is_alphabetic())
                || core.chars().count() <= 1
            {
                return false;
            }
            let all_lower = core.chars().all(|c| c.is_lowercase());
            let all_upper = core.chars().all(|c| c.is_uppercase());
            all_lower || all_upper || is_titlecase(core)
        }

        pub(crate) fn correct_word(&self, word: &str) -> Result<String, String> {
            let core = crate::casing::core(word);
            if !Self::should_spellcheck(word) {
                return Ok(word.to_string());
            }

            let input = self.nsstring_from_str(core)?;
            let range = NSRange {
                location: 0,
                length: unsafe { msg_send0(input, selector(b"length\0")) },
            };

            unsafe {
                let mut word_count: NSInteger = 0;
                let misspelled: NSRange = msg_send6(
                    self.checker,
                    selector(b"checkSpellingOfString:startingAt:language:wrap:inSpellDocumentWithTag:wordCount:\0"),
                    input,
                    0usize,
                    ptr::null_mut::<c_void>(),
                    false,
                    0isize,
                    &mut word_count as *mut NSInteger,
                );
                if misspelled.length == 0 {
                    return Ok(word.to_string());
                }

                let guesses: Id = msg_send4(
                    self.checker,
                    selector(b"guessesForWordRange:inString:language:inSpellDocumentWithTag:\0"),
                    range,
                    input,
                    ptr::null_mut::<c_void>(),
                    0isize,
                );
                if guesses.is_null() {
                    return Ok(word.to_string());
                }

                let count: NSUInteger = msg_send0(guesses, selector(b"count\0"));
                if count == 0 {
                    return Ok(word.to_string());
                }

                let first: Id = msg_send1(guesses, selector(b"objectAtIndex:\0"), 0usize);
                if first.is_null() {
                    return Ok(word.to_string());
                }

                let suggestion = rust_string_from_nsstring(first)?;
                if suggestion.is_empty() {
                    return Ok(word.to_string());
                }
                let corrected = normalize_suggestion(core, &suggestion);
                if corrected == core {
                    Ok(word.to_string())
                } else {
                    Ok(word.replacen(core, &corrected, 1))
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    /// Placeholder implementation so the crate compiles on non-macOS targets.
    pub(crate) struct SpellChecker;

    impl SpellChecker {
        pub(crate) fn new() -> Result<Self, String> {
            Err("--spellcheck is only supported on macOS".to_string())
        }

        pub(crate) fn correct_word(&self, word: &str) -> Result<String, String> {
            Ok(word.to_string())
        }
    }
}

pub(crate) use imp::SpellChecker;
