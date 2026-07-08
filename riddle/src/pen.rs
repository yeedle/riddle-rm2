//! Raw evdev pen input: the full digitizer, bypassing Qt's filtered view.
//! Gives us 0-4096 pressure, tilt, hover, and the eraser tip (BTN_TOOL_RUBBER),
//! at the hardware event rate.
//!
//! The device is grabbed (EVIOCGRAB) while the diary is open so xochitl
//! doesn't also react to the pen; released automatically on close/exit.

use std::io;
use std::os::fd::RawFd;

use crate::fb::{SCREEN_H, SCREEN_W};

// Digitizer axis ranges differ per device:
//   Paper Pro ("Elan marker input"): 11180 x 15340
//   reMarkable 2 ("Wacom I2C Digitizer"): 20966 x 15725
#[cfg(feature = "rm2")]
const DIGI_MAX_X: i32 = 20966;
#[cfg(feature = "rm2")]
const DIGI_MAX_Y: i32 = 15725;
#[cfg(not(feature = "rm2"))]
const DIGI_MAX_X: i32 = 11180;
#[cfg(not(feature = "rm2"))]
const DIGI_MAX_Y: i32 = 15340;
pub const MAX_PRESSURE: i32 = 4096;

// Minimum pressure to count as "writing". The Paper Pro used a hard-coded 40;
// the rm2's Wacom reports hover and has a noisier low end, so it needs a higher
// floor. Overridable at runtime with RIDDLE_PEN_MIN_PRESSURE for easy tuning.
#[cfg(feature = "rm2")]
pub const DEFAULT_MIN_PRESSURE: i32 = 120;
#[cfg(not(feature = "rm2"))]
pub const DEFAULT_MIN_PRESSURE: i32 = 40;

pub fn min_pressure() -> i32 {
    std::env::var("RIDDLE_PEN_MIN_PRESSURE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MIN_PRESSURE)
}

// On the reMarkable 2 the Wacom digitizer is mounted rotated 90° relative to
// the panel: its long axis (ABS_X, 0..20966) runs down the SCREEN HEIGHT and
// its short axis (ABS_Y, 0..15725) runs across the SCREEN WIDTH. So screen_x
// comes from raw_y and screen_y from raw_x. Empirically ABS_X grows bottom→top
// (opposite of the screen's top→bottom), so Y is inverted; X matches.
#[cfg(feature = "rm2")]
const INVERT_X: bool = false;
#[cfg(feature = "rm2")]
const INVERT_Y: bool = true;

// Size of one `struct input_event` as delivered by the kernel evdev ABI.
// The leading timestamp is two `long`s, so the struct is 16 bytes on 32-bit
// (reMarkable 2, ARMv7) and 24 bytes on 64-bit (Paper Pro, AArch64). The
// type/code/value fields follow the timestamp.
#[cfg(target_pointer_width = "64")]
pub const EV_SIZE: usize = 24;
#[cfg(target_pointer_width = "64")]
pub const EV_OFF: usize = 16;
#[cfg(not(target_pointer_width = "64"))]
pub const EV_SIZE: usize = 16;
#[cfg(not(target_pointer_width = "64"))]
pub const EV_OFF: usize = 8;

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;
const SYN_REPORT: u16 = 0;
const ABS_X: u16 = 0;
const ABS_Y: u16 = 1;
const ABS_PRESSURE: u16 = 24;
const BTN_TOOL_PEN: u16 = 320;
const BTN_TOOL_RUBBER: u16 = 321;
const BTN_TOUCH: u16 = 330;

const EVIOCGRAB: libc::c_ulong = 0x40044590;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
}

#[derive(Debug, Clone, Copy)]
pub struct PenSample {
    /// Screen coordinates.
    pub x: i32,
    pub y: i32,
    /// 0..4096
    pub pressure: i32,
    pub tool: Tool,
    pub touching: bool,
}

pub struct PenDevice {
    fd: RawFd,
    // Accumulated state between SYN_REPORTs.
    raw_x: i32,
    raw_y: i32,
    pressure: i32,
    tool: Tool,
    touching: bool,
    dirty: bool,
    // Diagnostics: when RIDDLE_PEN_DEBUG is set, log the first samples' raw and
    // mapped coordinates so we can calibrate the rm2 axis mapping empirically.
    dbg: bool,
    dbg_n: u32,
}

impl PenDevice {
    /// Find and grab the marker input device.
    pub fn open() -> io::Result<Self> {
        let path = find_marker_device()?;
        let cpath = std::ffi::CString::new(path.clone()).unwrap();
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let grab = unsafe { libc::ioctl(fd, EVIOCGRAB, 1i32) };
        if grab != 0 {
            eprintln!("riddle: warning: EVIOCGRAB failed ({}) — xochitl will also see the pen", io::Error::last_os_error());
        }
        eprintln!("riddle: pen device {path} opened (grabbed: {})", grab == 0);
        Ok(Self {
            fd,
            raw_x: 0,
            raw_y: 0,
            pressure: 0,
            tool: Tool::Pen,
            touching: false,
            dirty: false,
            dbg: std::env::var("RIDDLE_PEN_DEBUG").is_ok(),
            dbg_n: 0,
        })
    }

    #[allow(dead_code)]
    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }

    /// Drain all pending events; returns one sample per SYN_REPORT frame
    /// that changed state.
    pub fn drain(&mut self) -> Vec<PenSample> {
        let mut out = Vec::new();
        // input_event: struct timeval (2 longs) + type u16 + code u16 + value i32.
        // Size/offset are architecture-dependent (see EV_SIZE / EV_OFF).
        let mut buf = [0u8; 24 * 64];
        loop {
            let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for chunk in buf[..n as usize].chunks_exact(EV_SIZE) {
                let etype = u16::from_le_bytes(chunk[EV_OFF..EV_OFF + 2].try_into().unwrap());
                let code = u16::from_le_bytes(chunk[EV_OFF + 2..EV_OFF + 4].try_into().unwrap());
                let value = i32::from_le_bytes(chunk[EV_OFF + 4..EV_OFF + 8].try_into().unwrap());
                match (etype, code) {
                    (EV_ABS, ABS_X) => {
                        self.raw_x = value;
                        self.dirty = true;
                    }
                    (EV_ABS, ABS_Y) => {
                        self.raw_y = value;
                        self.dirty = true;
                    }
                    (EV_ABS, ABS_PRESSURE) => {
                        self.pressure = value;
                        self.dirty = true;
                    }
                    (EV_KEY, BTN_TOOL_PEN) if value == 1 => {
                        self.tool = Tool::Pen;
                    }
                    (EV_KEY, BTN_TOOL_RUBBER) => {
                        self.tool = if value == 1 { Tool::Eraser } else { Tool::Pen };
                    }
                    (EV_KEY, BTN_TOUCH) => {
                        self.touching = value == 1;
                        self.dirty = true;
                    }
                    (EV_SYN, SYN_REPORT) => {
                        if self.dirty {
                            self.dirty = false;
                            let (x, y) = map_to_screen(self.raw_x, self.raw_y);
                            if self.dbg && self.dbg_n < 150 {
                                self.dbg_n += 1;
                                eprintln!(
                                    "riddle: pen raw=({},{}) -> screen=({},{}) pressure={} touch={}",
                                    self.raw_x, self.raw_y, x, y, self.pressure, self.touching
                                );
                            }
                            out.push(PenSample {
                                x,
                                y,
                                pressure: self.pressure,
                                tool: self.tool,
                                touching: self.touching,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        out
    }
}

impl Drop for PenDevice {
    fn drop(&mut self) {
        unsafe {
            libc::ioctl(self.fd, EVIOCGRAB, 0i32);
            libc::close(self.fd);
        }
    }
}

fn find_marker_device() -> io::Result<String> {
    for i in 0..8 {
        let name_path = format!("/sys/class/input/event{i}/device/name");
        if let Ok(name) = std::fs::read_to_string(&name_path) {
            let n = name.to_lowercase();
            // "marker" = Paper Pro (Elan); "wacom" = reMarkable 2 digitizer.
            if n.contains("marker") || n.contains("wacom") {
                return Ok(format!("/dev/input/event{i}"));
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "no marker input device found"))
}

/// Map raw digitizer coordinates to screen pixels.
///
/// rm2: the Wacom panel is rotated 90° vs the display, so the axes are swapped
/// (screen_x from raw_y, screen_y from raw_x). Paper Pro maps straight through.
#[cfg(feature = "rm2")]
fn map_to_screen(raw_x: i32, raw_y: i32) -> (i32, i32) {
    let mut x = raw_y * (SCREEN_W as i32 - 1) / DIGI_MAX_Y;
    let mut y = raw_x * (SCREEN_H as i32 - 1) / DIGI_MAX_X;
    if INVERT_X {
        x = (SCREEN_W as i32 - 1) - x;
    }
    if INVERT_Y {
        y = (SCREEN_H as i32 - 1) - y;
    }
    (x, y)
}

#[cfg(not(feature = "rm2"))]
fn map_to_screen(raw_x: i32, raw_y: i32) -> (i32, i32) {
    let x = raw_x * (SCREEN_W as i32 - 1) / DIGI_MAX_X;
    let y = raw_y * (SCREEN_H as i32 - 1) / DIGI_MAX_Y;
    (x, y)
}
