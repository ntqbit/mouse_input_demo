use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use winapi::{
    shared::{
        hidusage::{HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC},
        minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM},
        windef::{HBRUSH, HICON, HMENU, HWND},
    },
    um::{
        winnt::LPCWSTR,
        winuser::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageA, GetMessageW,
            GetRawInputData, HRAWINPUT, MOUSE_MOVE_ABSOLUTE, MSG, PostQuitMessage, RAWINPUT,
            RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK,
            RIM_TYPEMOUSE, RegisterClassW, RegisterRawInputDevices, SW_SHOW, ShowWindow,
            TranslateMessage, UnregisterClassW, WM_DESTROY, WM_INPUT, WNDCLASSW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

fn get_raw_input_data(handle: HRAWINPUT) -> Option<RAWINPUT> {
    let mut data: RAWINPUT = unsafe { core::mem::zeroed() };
    let mut data_size = size_of::<RAWINPUT>() as u32;
    let header_size = size_of::<RAWINPUTHEADER>() as u32;

    let status = unsafe {
        GetRawInputData(
            handle,
            RID_INPUT,
            &mut data as *mut _ as _,
            &mut data_size,
            header_size,
        )
    };

    if status == u32::MAX || status == 0 {
        return None;
    }

    Some(data)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            0
        },
        WM_INPUT => {
            println!("wm input");
            if let Some(raw_input) = get_raw_input_data(lparam as HRAWINPUT) {
                if raw_input.header.dwType == RIM_TYPEMOUSE {
                    let me = unsafe { raw_input.data.mouse() };
                    let absolute = me.usFlags & MOUSE_MOVE_ABSOLUTE != 0;
                    let pos = (me.lLastX, me.lLastY);

                    println!(
                        "mouse event: flags={}, absolute={}, pos={:?}",
                        me.usFlags, absolute, pos
                    );
                }
            }
            0
        }
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

const WINDOW_NAME: &str = "mouse_input_demo";

pub fn register_raw_input_devices(hwnd: HWND) {
    let devices = [RAWINPUTDEVICE {
        usUsagePage: HID_USAGE_PAGE_GENERIC,
        usUsage: HID_USAGE_GENERIC_MOUSE,
        dwFlags: RIDEV_DEVNOTIFY | RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    }];

    let device_size = std::mem::size_of::<RAWINPUTDEVICE>() as u32;

    let res =
        unsafe { RegisterRawInputDevices(devices.as_ptr(), devices.len() as u32, device_size) };

    assert_eq!(res, 1);
}

fn main() {
    // Win32 window creation: https://www.jendrikillner.com/post/rust-game-part-2/

    let mut window_name: Vec<u16> = OsStr::new(WINDOW_NAME).encode_wide().collect();
    window_name.push(0);

    unsafe {
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: window_name.as_ptr(),
        };

        let error_code = RegisterClassW(&wc);

        assert!(error_code != 0, "failed to register the window class");

        let hwnd = CreateWindowExW(
            0,
            window_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            400,
            400,
            0 as HWND,
            0 as HMENU,
            wc.hInstance,
            std::ptr::null_mut(),
        );

        assert!(hwnd != (0 as HWND), "failed to open the window");

        ShowWindow(hwnd, SW_SHOW);

        register_raw_input_devices(hwnd);

        let mut msg: MSG = std::mem::zeroed();

        while GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        DestroyWindow(hwnd);
        UnregisterClassW(wc.lpszClassName, wc.hInstance);
    }
}
