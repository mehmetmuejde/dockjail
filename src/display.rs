//! Looking up displays by position and by hardware UUID.

use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation::uuid::CFUUID;
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::uuid::CFUUIDCreateString;
use core_graphics::display::CGDisplay;
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGRect};
use core_graphics::window;

use crate::ffi;

pub struct Display {
    pub uuid: String,
    pub frame: CGRect,
    pub is_main: bool,
    pub is_builtin: bool,
    pub id: u32,
}

/// All active displays, main display first, the rest ordered left to right.
pub fn list() -> Vec<Display> {
    let mut displays: Vec<Display> = CGDisplay::active_displays()
        .unwrap_or_default()
        .into_iter()
        .map(CGDisplay::new)
        .map(|d| Display {
            uuid: uuid_of(d.id),
            frame: d.bounds(),
            is_main: d.is_main(),
            is_builtin: d.is_builtin(),
            id: d.id,
        })
        .collect();
    displays.sort_by(|a, b| b.is_main.cmp(&a.is_main).then(a.frame.origin.x.total_cmp(&b.frame.origin.x)));
    displays
}

/// The display that contains `point` (global coordinates).
pub fn at(point: CGPoint) -> Option<u32> {
    let (ids, count) = CGDisplay::displays_with_point(point, 1).ok()?;
    ids.first().copied().filter(|_| count > 0)
}

/// Windows smaller than this (in points) are ignored when looking for the front window: some apps keep
/// tiny invisible windows on screen.
const MIN_WINDOW_SIZE: f64 = 50.0;

/// The display that shows the frontmost normal window. Right after a command is typed, that is the
/// terminal window it was typed in. Window bounds need no Screen Recording permission.
pub fn of_front_window() -> Option<u32> {
    let options = window::kCGWindowListOptionOnScreenOnly | window::kCGWindowListExcludeDesktopElements;
    let windows = window::copy_window_info(options, window::kCGNullWindowID)?;
    let (layer_key, bounds_key) = unsafe {
        (CFString::wrap_under_get_rule(window::kCGWindowLayer), CFString::wrap_under_get_rule(window::kCGWindowBounds))
    };

    // Ordered front to back; layer 0 holds normal windows, the menu bar and the Dock sit above it.
    for item in windows.iter() {
        let info: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_get_rule(*item as CFDictionaryRef) };
        let layer = info.find(&layer_key).and_then(|v| v.downcast::<CFNumber>()).and_then(|n| n.to_i64());
        if layer != Some(0) {
            continue;
        }
        let Some(bounds) = info.find(&bounds_key) else { continue };
        let mut frame = CGRect::default();
        if !unsafe { ffi::CGRectMakeWithDictionaryRepresentation(bounds.as_CFTypeRef().cast(), &mut frame) } {
            continue;
        }
        if frame.size.width < MIN_WINDOW_SIZE || frame.size.height < MIN_WINDOW_SIZE {
            continue;
        }
        let center = CGPoint::new(frame.origin.x + frame.size.width / 2.0, frame.origin.y + frame.size.height / 2.0);
        return at(center);
    }
    None
}

/// The display the mouse cursor is on.
pub fn under_cursor() -> Option<u32> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    at(CGEvent::new(source).ok()?.location())
}

/// The connected display with this UUID, if any.
pub fn with_uuid(uuid: &str) -> Option<u32> {
    CGDisplay::active_displays().ok()?.into_iter().find(|&id| uuid_of(id) == uuid)
}

/// Hardware UUID of a display; stays the same across reboots and port changes.
pub fn uuid_of(id: u32) -> String {
    unsafe {
        let uuid = ffi::CGDisplayCreateUUIDFromDisplayID(id);
        if uuid.is_null() {
            return format!("DISPLAY-{id}");
        }
        let uuid = CFUUID::wrap_under_create_rule(uuid);
        CFString::wrap_under_create_rule(CFUUIDCreateString(std::ptr::null(), uuid.as_concrete_TypeRef())).to_string()
    }
}
