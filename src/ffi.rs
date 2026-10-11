//! Raw bindings for the macOS APIs that the `core-graphics` and
//! `core-foundation` crates do not cover.

#![allow(non_upper_case_globals, non_snake_case)]

use std::ffi::c_void;

use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::mach_port::CFMachPortRef;
use core_foundation_sys::string::CFStringRef;
use core_foundation_sys::uuid::CFUUIDRef;
use core_graphics::geometry::CGRect;

// --- Accessibility (ApplicationServices) ---

pub type AXUIElementRef = *const c_void;
pub type AXError = i32;
pub type AXValueType = u32;

pub const kAXErrorSuccess: AXError = 0;
pub const kAXValueTypeCGPoint: AXValueType = 1;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub static kAXTrustedCheckOptionPrompt: CFStringRef;

    pub fn AXIsProcessTrusted() -> u8;
    pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    pub fn AXUIElementCreateApplication(pid: libc::pid_t) -> AXUIElementRef;
    pub fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    pub fn AXValueGetValue(value: CFTypeRef, the_type: AXValueType, value_ptr: *mut c_void) -> u8;
}

// --- Core Graphics ---

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CGRectMakeWithDictionaryRepresentation(dict: CFDictionaryRef, rect: *mut CGRect) -> bool;
}

#[link(name = "ColorSync", kind = "framework")]
unsafe extern "C" {
    pub fn CGDisplayCreateUUIDFromDisplayID(display: u32) -> CFUUIDRef;
}
