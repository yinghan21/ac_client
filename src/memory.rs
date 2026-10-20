use std::ffi::c_void;
use std::sync::OnceLock;
use hudhook::hooks::find_process_hwnd;
use windows_sys::s;
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA};
use windows_sys::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowA, GetClientRect, IsWindow};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, y
        }
    }
    pub fn to_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }
}

pub fn module_base() -> Option<usize> {
    let name = s!("ac_client.exe");
    let base = unsafe { GetModuleHandleA(name)};
    if base.is_null() {
        return None;
    }
    Some(base as usize)
}

pub fn is_readable(addr: usize, size: usize) -> bool {
    if addr == 0 || size == 0 {
        return false;
    }
    let mut a = addr;
    let mut mbi = unsafe { std::mem::zeroed::<MEMORY_BASIC_INFORMATION>()};
    let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    let Some(end) = addr.checked_add(size) else {return false};

    while a < end {
        let ret = unsafe { VirtualQuery(a as *const c_void, &mut mbi, mbi_size)};
        if ret == 0 {
            return false;
        }
        if mbi.State != MEM_COMMIT {
            return false;
        }
        if mbi.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0 {
            return false;
        }
        let region_end = (mbi.BaseAddress as usize).saturating_add(mbi.RegionSize);
        if region_end <= a {
            return false;
        }
        a = region_end;
    }
    true
}

pub fn read<T: Copy>(addr: usize) -> Option<T> {
    if !is_readable(addr, std::mem::size_of::<T>()) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(addr as *const T) })
}

pub fn world_to_screen(world: Vec3, matrix: &[f32; 16], width: f32, height: f32) -> Option<Vec2> {
    let w = world;
    let m = matrix;
    let x = w.x * m[0] + w.y * m[4] + w.z * m[8] + m[12];
    let y = w.x * m[1] + w.y * m[5] + w.z * m[9] + m[13];
    let w = w.x * m[3] + w.y * m[7] + w.z * m[11] + m[15];
    if !w.is_finite() || w < 0.001 {
        return None;
    }
    let ndc_x = x / w;
    let ndc_y = y / w;
    let screen_x = width * 0.5;
    let screen_y = height * 0.5;
    Some(Vec2::new(
        screen_x + ndc_x * screen_x,
        screen_y - ndc_y * screen_y,
    ))
}

pub fn find_game_window() -> Option<HWND> {
    let name = s!("AssaultCube");
    if let Some(hwnd) = find_process_hwnd() {
        return Some(hwnd.0 as HWND);
    }
    let hwnd = unsafe { FindWindowA(std::ptr::null(), name) };
    if hwnd.is_null() {
        return None;
    }
    Some(hwnd)
}

static GAME_HWND: OnceLock<isize> = OnceLock::new();
pub fn game_hwnd() -> Option<HWND> {
    if let Some(&raw) = GAME_HWND.get() {
        if unsafe { IsWindow(raw as HWND) } != 0 {
            return None;
        };
    };
    let hwnd = find_game_window()?;
    let _ = GAME_HWND.set(hwnd as isize);
    Some(hwnd)
}

pub fn game_client_size() -> Option<(f32, f32)> {
    let hwnd = find_game_window()?;
    let mut rect = unsafe { std::mem::zeroed::<RECT>() };
    let ok = unsafe { GetClientRect(hwnd, &mut rect) };
    if ok == 0 {
        return None;
    }
    let w = (rect.right - rect.left) as f32;
    let h = (rect.bottom - rect.top) as f32;
    if w < 0.0 || h < 0.0 {
        return None;
    }
    Some((w, h))
}