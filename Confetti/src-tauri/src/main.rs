// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use raw_window_handle::{RawWindowHandle, HasRawWindowHandle};

// Learn more about Tauri commands at https://v1.tauri.app/v1/guides/features/command
#[tauri::command]
fn set_transparent_clickthrough(window: tauri::Window) -> Result<(), String> {
    let _ = window.set_decorations(false);
    let _ = window.set_always_on_top(true);

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetLayeredWindowAttributes, GWL_EXSTYLE,
            WS_EX_LAYERED, WS_EX_TRANSPARENT,
        };

        if let RawWindowHandle::Win32(handle) = window.raw_window_handle() {
            // hwnd is a *mut c_void; wrap for windows crate
            let hwnd = HWND(handle.hwnd as isize);
            unsafe {
                // get current ex style
                let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;
                let new = ex | (WS_EX_LAYERED.0 as isize) | (WS_EX_TRANSPARENT.0 as isize);
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new);
                // use LAYERED_WINDOW_ATTRIBUTES_FLAGS wrapper to match windows 0.39 signature
                SetLayeredWindowAttributes(
                    hwnd,
                    0u32,
                    255u8,
                    windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS(0x2u32),
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitWindowHandle;
        use cocoa::appkit::{NSWindow, NSColor};
        use cocoa::base::{id, YES, NO};
        use objc::runtime::Object;
        if let RawWindowHandle::AppKit(handle) = window.raw_window_handle() {
            let ns_window_ptr = handle.ns_window as *mut Object;
            if !ns_window_ptr.is_null() {
                unsafe {
                    let ns_window: id = ns_window_ptr as id;
                    let ns_win = NSWindow::from_ptr(ns_window);
                    ns_win.setOpaque_(NO);
                    ns_win.setBackgroundColor_(NSColor::clearColor());
                    ns_win.setIgnoresMouseEvents_(YES);
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn start_confetti() -> Result<(), String> {
    // no-op backend hook so invoke('start_confetti') doesn't fail
    println!("start_confetti invoked");
    Ok(())
}

#[tauri::command]
fn clear_transparent_clickthrough(window: tauri::Window) -> Result<(), String> {
    // restore native window flags / behavior per platform
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
        };

        if let RawWindowHandle::Win32(handle) = window.raw_window_handle() {
            let hwnd = HWND(handle.hwnd as isize);
            unsafe {
                let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as isize;
                let remove_mask = (WS_EX_LAYERED.0 as isize) | (WS_EX_TRANSPARENT.0 as isize);
                let new = ex & !remove_mask;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSWindow, NSColor};
        use cocoa::base::{id, YES, NO};
        use objc::runtime::Object;
        use raw_window_handle::AppKitWindowHandle;

        if let RawWindowHandle::AppKit(handle) = window.raw_window_handle() {
            let ns_window_ptr = handle.ns_window as *mut Object;
            if !ns_window_ptr.is_null() {
                unsafe {
                    let ns_window: id = ns_window_ptr as id;
                    let ns_win = NSWindow::from_ptr(ns_window);
                    ns_win.setIgnoresMouseEvents_(NO);
                    ns_win.setOpaque_(YES);
                    ns_win.setBackgroundColor_(NSColor::whiteColor());
                }
            }
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_transparent_clickthrough,
            clear_transparent_clickthrough,
            start_confetti
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
