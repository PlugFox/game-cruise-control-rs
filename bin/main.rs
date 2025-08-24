// Do not delete this:
//#![cfg_attr(not(target_os = "windows"), allow(unused_imports, unused_variables))]
//use rand::Rng;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::Media::Speech::{ISpVoice, SPF_DEFAULT, SpVoice};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, /* KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MapVirtualKeyW, */
    SendInput, VIRTUAL_KEY,
};

#[cfg(target_os = "windows")]
const VK_PAGE_UP: i32 = 0x21; // VK_PRIOR (Page Up)
#[cfg(target_os = "windows")]
const VK_PAGE_DOWN: i32 = 0x22; // VK_NEXT (Page Down)
#[cfg(target_os = "windows")]
const VK_W: i32 = 0x57; // VK_W
#[cfg(target_os = "windows")]
const VK_S: i32 = 0x53; // VK_S
#[cfg(target_os = "windows")]
const VK_SPACE: i32 = 0x20; // VK_SPACE

// Check is windows, if not - show error message and close program
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This program currently supports only on Windows.");
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn main() {
    println!("Start cruise on Page Up. Stops on S or Space. Ctrl+C to exit.");

    // Cruise control state
    let cruise_active = Arc::new(AtomicBool::new(false));

    // Start overlay thread (Windows only)
    #[cfg(target_os = "windows")]
    {
        let state = Arc::clone(&cruise_active);
        thread::spawn(move || {
            use game_cc::overlay::run_overlay_window;
            run_overlay_window(state);
        });
    }

    loop {
        // Current physical key states
        let pgup_down = is_key_down(VK_PAGE_UP);
        let pgdown_down = is_key_down(VK_PAGE_DOWN);
        let w_down = is_key_down(VK_W);
        let s_down = is_key_down(VK_S);
        let space_down = is_key_down(VK_SPACE);

        if cruise_active.load(Ordering::Relaxed) {
            if s_down || space_down || pgdown_down {
                // Stop cruise control on S or Space
                cruise_active.store(false, Ordering::Relaxed);
                cruise_control_stop(true);
            } else if !w_down {
                // If W is released, start cruise control again
                cruise_control_start(false);
            }
        } else if pgup_down {
            // Start cruise control
            cruise_control_start(true);
            cruise_active.store(true, Ordering::Relaxed);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

/// Send a key event using virtual key codes.
#[cfg(target_os = "windows")]
fn send_key_vk(vk: u16, down: bool) {
    let flags: KEYBD_EVENT_FLAGS = if down {
        KEYBD_EVENT_FLAGS(0)
    } else {
        KEYEVENTF_KEYUP
    };
    let ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        wScan: 0,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki },
    };
    unsafe {
        let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Start cruise control.
fn cruise_control_start(speak: bool) {
    const VK_W_U16: u16 = 0x57;
    //send_key_scancode(VK_W_U16, true);
    send_key_vk(VK_W_U16, true);
    if speak {
        speak_async("Cruise control enabled");
    }
}

/// Stop cruise control.
fn cruise_control_stop(speak: bool) {
    const VK_W_U16: u16 = 0x57;
    send_key_vk(VK_W_U16, false);
    //send_key_scancode(VK_W_U16, false);
    if speak {
        speak_async("Cruise control disabled");
    }
}

/// Check if a virtual key is currently pressed.
#[cfg(target_os = "windows")]
fn is_key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16) & 0x8000 != 0 }
}

/// Speak a text asynchronously.
#[cfg(target_os = "windows")]
fn speak_async(text: &str) {
    let text = text.to_string();
    // Spawn a short-lived thread so we don't block the input loop
    thread::spawn(move || unsafe {
        // Initialize COM for this thread
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return;
        }
        // Create SAPI SpVoice
        let voice: Result<ISpVoice, _> = CoCreateInstance(&SpVoice, None, CLSCTX_ALL);
        if let Ok(v) = voice {
            // Convert to wide string (UTF-16) with null terminator
            use windows::core::PCWSTR;
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let pw = PCWSTR(wide.as_ptr());
            let _ = v.Speak(pw, SPF_DEFAULT.0 as u32, None);
        }
        CoUninitialize();
    });
}
