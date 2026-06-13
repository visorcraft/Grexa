// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CString;

unsafe extern "C" {
    fn grexa_set_icon_theme(name: *const std::ffi::c_char);
}

/// Force Qt/Kirigami to resolve `icon.name` / `Kirigami.Icon` sources against
/// the named theme. Used in the AppImage build, which does not inherit the
/// host's Breeze icon theme.
pub fn set_icon_theme(name: &str) {
    let c_name = CString::new(name).expect("icon theme name contains null byte");
    unsafe { grexa_set_icon_theme(c_name.as_ptr()) };
}
