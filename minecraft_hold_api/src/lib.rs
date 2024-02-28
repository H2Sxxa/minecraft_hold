use std::{collections::HashMap, ffi::CString};

use serde::Serialize;
use serde_json::to_string;
use sharedlib::{Func, Lib, Symbol};
use windows::Win32::{
    Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_DEBUG_NAME,
        SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        ProcessStatus::{EmptyWorkingSet, GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        Threading::{
            GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
            PROCESS_SET_QUOTA, PROCESS_SUSPEND_RESUME, PROCESS_VM_READ,
        },
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, ShowWindow, SW_HIDE, SW_SHOW,
    },
};
use wmi::Variant;

#[derive(Debug, Serialize)]
pub struct MinecraftInfo {
    pub pid: u32,
    pub name: String,
}

#[no_mangle]
pub extern "C" fn get_dbg_privilege() {
    let mut token = HANDLE::default();
    let mut id = LUID::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES,
            &mut token,
        )
        .unwrap();

        LookupPrivilegeValueW(None, SE_DEBUG_NAME, &mut id).unwrap();
        AdjustTokenPrivileges(
            token,
            false,
            Some(&TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: id,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            }),
            0,
            None,
            None,
        )
        .unwrap();
    }
}

#[no_mangle]
pub extern "C" fn hide_window(pid: u32) {
    extern "system" fn enum_window(window: HWND, lprm: LPARAM) -> BOOL {
        unsafe {
            let mut target_pid: u32 = 0;
            GetWindowThreadProcessId(window, Some(&mut target_pid));
            if target_pid == lprm.0 as u32 {
                ShowWindow(window, SW_HIDE);
            }
        }
        true.into()
    }
    unsafe {
        EnumWindows(Some(enum_window), LPARAM(pid as isize)).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn show_window(pid: u32) {
    extern "system" fn enum_window(window: HWND, lprm: LPARAM) -> BOOL {
        unsafe {
            let mut target_pid: u32 = 0;
            GetWindowThreadProcessId(window, Some(&mut target_pid));
            if target_pid == lprm.0 as u32 {
                ShowWindow(window, SW_SHOW);
            }
        }

        true.into()
    }
    unsafe {
        EnumWindows(Some(enum_window), LPARAM(pid as isize)).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

#[no_mangle]
pub extern "C" fn suspend_process(pid: u32) {
    unsafe {
        let hobj = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid).unwrap();
        let lib = Lib::new("ntdll.dll").unwrap();
        let suspend: Func<extern "C" fn(HANDLE)> = lib.find_func("NtSuspendProcess").unwrap();
        suspend.get()(hobj);
        CloseHandle(hobj).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn resume_process(pid: u32) {
    unsafe {
        let hobj = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid).unwrap();
        let lib = Lib::new("ntdll.dll").unwrap();
        let suspend: Func<extern "C" fn(HANDLE)> = lib.find_func("NtResumeProcess").unwrap();
        suspend.get()(hobj);
        CloseHandle(hobj).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn clean_working_set(pid: u32) {
    unsafe {
        let hobj = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid).unwrap();
        EmptyWorkingSet(hobj).unwrap();
        CloseHandle(hobj).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn get_process_memory_info(pid: u32) -> usize {
    let mut count = PROCESS_MEMORY_COUNTERS::default();
    unsafe {
        let hobj = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).unwrap();
        GetProcessMemoryInfo(hobj, &mut count, count.cb).unwrap();
    }
    count.WorkingSetSize
}

#[no_mangle]
pub fn find_minecrafts() -> Vec<MinecraftInfo> {
    use wmi::{COMLibrary, WMIConnection};
    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con).unwrap();

    let wmi_results: Vec<HashMap<String, Variant>> = wmi_con
        .raw_query(
            r#"SELECT Name,ProcessID,CommandLine FROM Win32_Process WHERE name like '%java%'"#,
        )
        .unwrap();
    wmi_results
        .iter()
        .filter(|map| {
            if let Variant::String(value) = &map["CommandLine"] {
                return value.contains("minecraft");
            }
            false
        })
        .map(|e| {
            let Variant::UI4(pid) = &e["ProcessId"] else {
                todo!()
            };
            let Variant::String(name) = &e["Name"] else {
                todo!()
            };
            MinecraftInfo {
                pid: pid.clone(),
                name: name.clone(),
            }
        })
        .collect()
}

#[no_mangle]
pub fn find_minecrafts_native() -> CString {
    let info = find_minecrafts();
    let data = to_string(&info).unwrap();
    CString::new(data).unwrap()
}

#[no_mangle]
pub fn suspend_minecraft(pid: u32) {
    hide_window(pid);
    clean_working_set(pid);
    suspend_process(pid);
}

#[no_mangle]
pub fn resume_minecraft(pid: u32) {
    resume_process(pid);
    show_window(pid)
}

#[test]
fn test_wmi() {
    println!("{:?}", find_minecrafts());
}

#[test]
fn hide_mc() {
    find_minecrafts().iter().for_each(|v| hide_window(v.pid));
}

#[test]
fn show_mc() {
    find_minecrafts().iter().for_each(|v| show_window(v.pid));
}

#[test]
fn suspend_test() {
    find_minecrafts().iter().for_each(|v| {
        println!("{}", v.pid);

        suspend_minecraft(v.pid);
    });
}

#[test]
fn resume_test() {
    find_minecrafts().iter().for_each(|v| {
        println!("{}", v.pid);
        resume_minecraft(v.pid);
    });
}
