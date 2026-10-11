//! The Accessibility API: window elements, their attributes, and the trust it needs.

use std::ffi::c_void;
use std::ptr::{self, NonNull};

use objc2_application_services::{
    AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElement, AXValue, AXValueType,
    kAXTrustedCheckOptionPrompt,
};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFRetained, CFString, CFType, CGPoint, CGSize,
};
use views_platform::WindowId;

unsafe extern "C" {
    /// Private, and relied on by every macOS window manager: the CoreGraphics window
    /// number of an Accessibility window element.
    fn _AXUIElementGetWindow(element: &AXUIElement, number: *mut u32) -> AXError;
}

/// Asks the system whether this process may use the Accessibility API, and shows the
/// user the prompt to grant it if not.
pub fn request_trust() -> bool {
    // SAFETY: reading a constant the framework exports.
    let prompt = unsafe { kAXTrustedCheckOptionPrompt };
    let options =
        CFDictionary::<CFString, CFBoolean>::from_slices(&[prompt], &[CFBoolean::new(true)]);
    // SAFETY: the options are the documented key with a boolean.
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) }
}

/// Whether this process may use the Accessibility API.
pub fn is_trusted() -> bool {
    // SAFETY: no preconditions.
    unsafe { AXIsProcessTrusted() }
}

/// A window is named by its app's process id and its CoreGraphics window number, which
/// is all it takes to find its element again.
pub fn id(window: &AXUIElement) -> Option<WindowId> {
    let mut pid = 0;
    // SAFETY: `pid` is a valid out pointer.
    unsafe { window.pid(NonNull::from(&mut pid)) };
    Some(WindowId((pid as u32 as usize) << 32 | number(window)? as usize))
}

/// The windows of the app with process id `pid`.
pub fn windows(pid: i32) -> Vec<CFRetained<AXUIElement>> {
    // SAFETY: an unknown pid yields an element whose every query fails.
    let app = unsafe { AXUIElement::new_application(pid) };
    let Some(windows) =
        attribute(&app, "AXWindows").and_then(|value| value.downcast::<CFArray>().ok())
    else {
        return Vec::new();
    };
    // SAFETY: the value of `AXWindows` is an array of elements.
    unsafe { windows.cast_unchecked::<AXUIElement>() }.to_vec()
}

/// The element of `window`, if its app still has it.
pub fn element(window: WindowId) -> Option<CFRetained<AXUIElement>> {
    let number = window.0 as u32;
    windows(pid(window)).into_iter().find(|element| self::number(element) == Some(number))
}

/// The process id of the app that owns `window`.
pub fn pid(window: WindowId) -> i32 {
    (window.0 >> 32) as u32 as i32
}

pub fn string(element: &AXUIElement, name: &str) -> Option<String> {
    Some(attribute(element, name)?.downcast::<CFString>().ok()?.to_string())
}

pub fn flag(element: &AXUIElement, name: &str) -> Option<bool> {
    Some(attribute(element, name)?.downcast::<CFBoolean>().ok()?.as_bool())
}

pub fn set_position(element: &AXUIElement, position: CGPoint) -> AXError {
    set(element, "AXPosition", AXValueType::CGPoint, position)
}

pub fn set_size(element: &AXUIElement, size: CGSize) -> AXError {
    set(element, "AXSize", AXValueType::CGSize, size)
}

fn number(window: &AXUIElement) -> Option<u32> {
    let mut number = 0;
    // SAFETY: `number` is a valid out pointer.
    let error = unsafe { _AXUIElementGetWindow(window, &mut number) };
    (error == AXError::Success).then_some(number)
}

fn attribute(element: &AXUIElement, name: &str) -> Option<CFRetained<CFType>> {
    let mut value = ptr::null();
    // SAFETY: `value` is a valid out pointer.
    let error = unsafe {
        element.copy_attribute_value(&CFString::from_str(name), NonNull::from(&mut value))
    };
    if error != AXError::Success {
        return None;
    }
    // SAFETY: a successful copy returns an owned reference.
    NonNull::new(value.cast_mut()).map(|value| unsafe { CFRetained::from_raw(value) })
}

fn set<T>(element: &AXUIElement, name: &str, kind: AXValueType, mut value: T) -> AXError {
    // SAFETY: `kind` describes `T`.
    let Some(value) = (unsafe { AXValue::new(kind, NonNull::from(&mut value).cast::<c_void>()) })
    else {
        return AXError::IllegalArgument;
    };
    // SAFETY: `AXPosition` and `AXSize` take an `AXValue` of that kind.
    unsafe { element.set_attribute_value(&CFString::from_str(name), &value) }
}
