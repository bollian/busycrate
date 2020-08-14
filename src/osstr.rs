/* This Source Code Form is subject to the terms of the Mozilla Public
 *  * License, v. 2.0. If a copy of the MPL was not distributed with this
 *   * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::OsStr;

/// Helper methods for working with OsStr (and OsString which `Deref`s to it).
///
/// This would be named OsStrExt if it weren't for the existence of
/// `std::os::{unix,windows}::ffi::OsStrExt`.
pub trait OsStrHelperExt {
    fn ends_with(&self, suffix: &str) -> bool;
    fn starts_with(&self, prefix: &str) -> bool;
}

impl OsStrHelperExt for OsStr {
    #[cfg(unix)]
    fn ends_with(&self, suffix: &str) -> bool {
        use std::os::unix::ffi::OsStrExt;

        if suffix.len() > self.len() {
            return false;
        }

        let suffix_bytes = suffix.as_bytes();
        let self_bytes = self.as_bytes();
        let self_bytes = &self_bytes[self_bytes.len() - suffix_bytes.len()..];
        return self_bytes == suffix_bytes;
    }

    #[cfg(unix)]
    fn starts_with(&self, prefix: &str) -> bool {
        use std::os::unix::ffi::OsStrExt;

        if prefix.len() > self.len() {
            return false;
        }

        let prefix_bytes = prefix.as_bytes();
        let self_bytes = self.as_bytes();
        let self_bytes = &self_bytes[..prefix_bytes.len()];
        return self_bytes == prefix_bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn empty() {
        let os_str = OsStr::new("");
        assert!(os_str.ends_with(""));
    }

    #[test]
    fn empty_suffix() {
        let empty = "";
        let filled = "text";

        assert!(OsStr::new(filled).ends_with(empty));
        assert!(!OsStr::new(empty).ends_with(filled));
    }

    #[test]
    fn larger_suffix() {
        let os_str = OsStr::new("text");
        assert!(!os_str.ends_with("suffix"));
    }

    #[test]
    fn equal_strings_suffixes() {
        let first = "text";
        let second = "text";

        assert!(OsStr::new(first).ends_with(second));
        assert!(OsStr::new(second).ends_with(first));
    }

    #[test]
    fn unequal_suffix() {
        let larger = "some text";
        let suffix = "text";

        assert!(OsStr::new(larger).ends_with(suffix));
        assert!(!OsStr::new(suffix).ends_with(larger));
    }

    #[cfg(unix)]
    #[test]
    fn invalid_utf8() {
        use std::os::unix::ffi::OsStrExt;

        let text = b"text\xff";
        let suffix = "ext";

        assert!(!OsStr::from_bytes(text).ends_with(suffix));

        // invalid thanks to the type system
        // assert!(!OsStr::new(suffix).ends_with(text));
    }
}
