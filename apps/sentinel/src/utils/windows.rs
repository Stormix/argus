use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

#[derive(Debug)]
pub struct Application {
    name: String,
    pid: u32,
    process_name: String,
    window_title: String,
    is_focused: bool,
}

pub fn get_open_applications() -> Result<Vec<Application>, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut applications = Vec::new();
    let foreground_window = unsafe { GetForegroundWindow() };

    for (pid, process) in sys.processes() {
        if let Some(window_title) = get_window_title_for_pid(pid.as_u32()) {
            let window_handle = get_window_for_pid(pid.as_u32());
            applications.push(Application {
                name: process.name().to_string(),
                pid: pid.as_u32(),
                process_name: process.name().to_string(),
                window_title,
                is_focused: unsafe {
                    window_handle.map_or(false, |hwnd| hwnd == foreground_window)
                },
            });
        }
    }

    Ok(applications)
}

fn get_window_title_for_pid(pid: u32) -> Option<String> {
    let hwnd = get_window_for_pid(pid)?;
    Some(get_window_text(hwnd))
}

fn get_window_for_pid(pid: u32) -> Option<HWND> {
    // Reuse existing find_window_by_pid implementation
    find_window_by_pid(pid)
}

fn find_window_by_pid(target_pid: u32) -> Option<HWND> {
    struct EnumWindowsData {
        target_pid: u32,
        hwnd: Option<HWND>,
    }

    extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let data = &mut *(lparam.0 as *mut EnumWindowsData);
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            if pid == data.target_pid {
                data.hwnd = Some(hwnd);
                BOOL(0) // Stop enumeration
            } else {
                BOOL(1) // Continue enumeration
            }
        }
    }

    let mut data = EnumWindowsData {
        target_pid,
        hwnd: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_window_callback),
            LPARAM((&mut data as *mut EnumWindowsData) as isize),
        );
    }

    data.hwnd
}

fn get_window_text(hwnd: HWND) -> String {
    let mut text = vec![0u16; 512];
    unsafe {
        let len = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len as usize])
    }
}
