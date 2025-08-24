// Do not delete this:
//#![cfg_attr(not(target_os = "windows"), allow(unused_imports, unused_variables))]
//use rand::Rng;

use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, MapVirtualKeyW, SendInput, VIRTUAL_KEY,
};
#[cfg(target_os = "windows")]
const VK_PAGE_UP: i32 = 0x21; // VK_PRIOR (Page Up)
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
    println!("Page Up: start/stop cruise. Stop also on W, S, or Space. Ctrl+C to exit.");

    // Simple edge-detection for relevant keys
    let mut prev_pgup = false;
    let mut prev_w = false;
    let mut prev_s = false;
    let mut prev_space = false;

    // Cruise control state
    let mut cruise_active = false; // we have logically pressed W via SendInput (no matching key up yet)
    let mut pending_release = false; // we owe a key up for our synthetic W when safe

    loop {
        // Current physical key states
        let pgup_down = is_key_down(VK_PAGE_UP);
        let w_down = is_key_down(VK_W);
        let s_down = is_key_down(VK_S);
        let space_down = is_key_down(VK_SPACE);

        // Rising edges
        let pgup_edge = pgup_down && !prev_pgup;
        let w_edge = w_down && !prev_w;
        let s_edge = s_down && !prev_s;
        let space_edge = space_down && !prev_space;

        // Start/Stop logic
        if pgup_edge {
            if !cruise_active && !pending_release {
                // Start cruise: press W down synthetically if user isn't already holding W
                if !w_down {
                    cruise_control_start();
                    cruise_active = true;
                } else {
                    // User already holds W; consider this a no-op start
                    cruise_active = false;
                    pending_release = false;
                }
            } else {
                // Stop cruise on Page Up
                if cruise_active {
                    pending_release = true;
                }
                cruise_active = false;
            }
        }

        // Other stop keys: W, S, Space
        if cruise_active && (w_edge || s_edge || space_edge) {
            // Mark that we should release our synthetic W, but only when the physical W is not held
            pending_release = true;
            cruise_active = false;
        }

        // Perform deferred release when safe (physical W not down)
        if pending_release && !w_down {
            cruise_control_stop();
            pending_release = false;
        }

        // Update previous states
        prev_pgup = pgup_down;
        prev_w = w_down;
        prev_s = s_down;
        prev_space = space_down;

        thread::sleep(Duration::from_millis(10));
    }
}

/// Maps a virtual key code to its scan code.
#[cfg(target_os = "windows")]
fn vk_scan(vk: u16) -> u16 {
    unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16 }
}

#[cfg(target_os = "windows")]
fn send_key(vk: u16, down: bool) {
    let flags: KEYBD_EVENT_FLAGS = if down {
        KEYBD_EVENT_FLAGS(0)
    } else {
        KEYEVENTF_KEYUP
    };
    let scan = vk_scan(vk);
    let ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        wScan: scan,
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

fn cruise_control_start() {
    const VK_W_U16: u16 = 0x57;
    send_key(VK_W_U16, true);
}

fn cruise_control_stop() {
    const VK_W_U16: u16 = 0x57;
    send_key(VK_W_U16, false);
}

#[cfg(target_os = "windows")]
fn is_key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16) & 0x8000 != 0 }
}
