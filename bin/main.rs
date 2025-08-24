#![cfg_attr(not(target_os = "windows"), allow(unused_imports, unused_variables))]
use rand::Rng;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_RETURN,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{MAPVK_VK_TO_VSC, MapVirtualKeyW};

const PAGE_UP_KEY: i32 = 0x22_i32;

// Check is windows, if not - show error message and close program
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This program currently supports only on Windows.");
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
fn main() {
    println!("Press Page Up to start cruise control. WASD to stop. Press Ctrl+C to exit.");

    // To avoid repeated triggers when key is held, use a simple debounce.
    let armed = Arc::new(AtomicBool::new(true));
    let armed_listener = armed.clone();
    let armed_rearm = armed.clone();

    // Spawn a rearm thread to periodically allow next trigger after a short cooldown
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(120));
            armed_rearm.store(true, Ordering::Relaxed);
        }
    });

    // Polling Page Up key
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
        loop {
            let pressed: bool;
            pressed = unsafe { GetAsyncKeyState(PAGE_UP_KEY) } as u16 & 0x8000 != 0;
            if pressed {
                cruise_control_start();
                /* if armed_listener.swap(false, Ordering::Relaxed) {
                    cruise_control_start();
                } */
            }
            thread::sleep(Duration::from_millis(10));
        }
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
    const VK_W: u16 = 0x57;
    send_key(VK_W, true);
}

fn cruise_control_stop() {
    const VK_W: u16 = 0x57;
    send_key(VK_W, false);
}
