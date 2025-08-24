// Do not delete this:
//#![cfg_attr(not(target_os = "windows"), allow(unused_imports, unused_variables))]
//use rand::Rng;

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::DwmFlush;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::HGDIOBJ;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::*;
#[cfg(target_os = "windows")]
use windows::Win32::Media::Speech::{ISpVoice, SPF_DEFAULT, SpVoice};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, /* KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MapVirtualKeyW, */
    SendInput, VIRTUAL_KEY,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;
#[cfg(target_os = "windows")]
use windows::core::w;
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

/*
/// Maps a virtual key code to its scan code.
#[cfg(target_os = "windows")]
fn vk_scan(vk: u16) -> u16 {
    unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16 }
} */

/*
/// Send a key event using scan codes.
#[cfg(target_os = "windows")]
fn send_key_scancode(vk: u16, down: bool) {
    let mut flags: KEYBD_EVENT_FLAGS = KEYEVENTF_SCANCODE;
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }
    let scan = vk_scan(vk);
    let ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0),
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
} */

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

// ===================== Overlay Window (Windows only) =====================
#[cfg(target_os = "windows")]
fn run_overlay_window(state: Arc<AtomicBool>) {
    // Make state available to wndproc safely
    let _ = OVERLAY_STATE.set(state);

    let h_module = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let h_instance = HINSTANCE(h_module.0);

    // Register window class
    let class_name = w!("CC_Status_Overlay_Class");
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(overlay_wnd_proc),
        hInstance: h_instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        lpszClassName: class_name,
        ..Default::default()
    };
    let atom = unsafe { RegisterClassW(&wc) };
    if atom == 0 {
        return;
    }

    // Create layered, topmost, transparent, click-through window
    let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW; // toolwindow avoids taskbar
    let style = WS_POPUP;

    let hwnd = unsafe {
        CreateWindowExW(
            ex_style,
            class_name,
            w!("CC Status Overlay"),
            style,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(h_instance),
            None,
        )
        .unwrap()
    };

    // Make full-screen sized to current primary monitor
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    unsafe {
        let _ = MoveWindow(hwnd, 0, 0, width, height, true);
    };

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);
    }

    // Message loop with event-driven repaint
    let mut msg = MSG::default();
    let mut last_state = get_state_on();
    loop {
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                return;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }

        // Repaint only when state toggles
        let current = get_state_on();
        if current != last_state {
            let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
            last_state = current;
        }
        // Block until the Desktop Window Manager has finished composition
        let _ = unsafe { DwmFlush() };
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_CREATE => {
            // Use color key transparency: paint background black, make black transparent
            let _ = unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_COLORKEY) };
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1), // prevent flicker
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            // Fill with black (the color key) to keep background transparent
            let mut client = RECT::default();
            let _ = unsafe { GetClientRect(hwnd, &mut client) };
            let black_brush_obj = unsafe { GetStockObject(BLACK_BRUSH) };
            let black_brush = HBRUSH(black_brush_obj.0);
            unsafe { FillRect(hdc, &client, black_brush) };

            // Compute box in top-right
            let padding = 16;
            let box_w = 120;
            let box_h = 48;
            let left = client.right - box_w - padding;
            let top = client.top + padding;
            let rect = RECT {
                left,
                top,
                right: left + box_w,
                bottom: top + box_h,
            };

            // Draw hollow rectangle (frame) with green/red depending on state
            let on = get_state_on();
            let color = if on { rgb(0, 255, 0) } else { rgb(255, 0, 0) };
            let stock_null_brush = unsafe { GetStockObject(HOLLOW_BRUSH) };
            let old_brush = unsafe { SelectObject(hdc, stock_null_brush) };
            let pen = unsafe { CreatePen(PS_SOLID, 2, color) };
            let old_pen = unsafe { SelectObject(hdc, HGDIOBJ(pen.0)) };
            let _ = unsafe { Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom) };

            // Text: On/Off centered
            let text = if on { "On" } else { "Off" };
            unsafe {
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, color);
            }
            let mut text_rect = rect;
            let mut wide = to_wide(text);
            unsafe {
                DrawTextW(
                    hdc,
                    &mut wide,
                    &mut text_rect,
                    DT_CENTER | DT_SINGLELINE | DT_VCENTER,
                )
            };

            // restore/select cleanup
            unsafe {
                SelectObject(hdc, old_pen);
                SelectObject(hdc, old_brush);
                let _ = DeleteObject(HGDIOBJ(pen.0));
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_os = "windows")]
static OVERLAY_STATE: OnceLock<Arc<AtomicBool>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn get_state_on() -> bool {
    OVERLAY_STATE
        .get()
        .map(|a| a.load(Ordering::Relaxed))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}
