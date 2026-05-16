use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::{COPY_FROM_PARENT, NONE};

#[derive(Clone, Copy)]
pub struct Monitor {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

pub struct DisplaySetup {
    pub conn: RustConnection,
    pub window: Window,
    pub gcontext: Gcontext,
    pub width: u16,
    pub height: u16,
    pub monitors: Vec<Monitor>,
}

impl DisplaySetup {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = RustConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;

        let monitors = detect_monitors(&conn, root, width, height);
        println!("Detected {} monitor(s).", monitors.len());

        let win_id = conn.generate_id()?;
        let win_aux = CreateWindowAux::new()
            .background_pixel(screen.black_pixel)
            .override_redirect(1)
            .event_mask(EventMask::KEY_PRESS | EventMask::KEY_RELEASE);

        conn.create_window(
            COPY_FROM_PARENT as u8,
            win_id,
            root,
            0,
            0,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &win_aux,
        )?;

        let gc_id = conn.generate_id()?;
        conn.create_gc(gc_id, win_id, &CreateGCAux::new())?;

        conn.map_window(win_id)?;
        conn.flush()?;

        grab_inputs(&conn, win_id);

        Ok(Self {
            conn,
            window: win_id,
            gcontext: gc_id,
            width,
            height,
            monitors,
        })
    }
}

fn detect_monitors(conn: &RustConnection, root: Window, width: u16, height: u16) -> Vec<Monitor> {
    let mut monitors = Vec::new();

    if let Ok(res_cookie) = randr::get_screen_resources_current(conn, root) {
        if let Ok(res) = res_cookie.reply() {
            for &crtc in &res.crtcs {
                if let Ok(info_cookie) = randr::get_crtc_info(conn, crtc, res.config_timestamp) {
                    if let Ok(info) = info_cookie.reply() {
                        if info.width > 0 && info.mode != 0 {
                            monitors.push(Monitor {
                                x: info.x,
                                y: info.y,
                                w: info.width,
                                h: info.height,
                            });
                        }
                    }
                }
            }
        }
    }

    if monitors.is_empty() {
        monitors.push(Monitor { x: 0, y: 0, w: width, h: height });
    }

    monitors
}

fn grab_inputs(conn: &RustConnection, window: Window) {
    for i in 0..50u32 {
        let kb = conn.grab_keyboard(
            true,
            window,
            Time::CURRENT_TIME,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        );
        let ptr = conn.grab_pointer(
            true,
            window,
            EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            window,
            NONE,
            Time::CURRENT_TIME,
        );

        if kb.is_ok() && ptr.is_ok() {
            let kb_status = kb
                .unwrap()
                .reply()
                .map(|r| r.status)
                .unwrap_or(GrabStatus::ALREADY_GRABBED);
            let ptr_status = ptr
                .unwrap()
                .reply()
                .map(|r| r.status)
                .unwrap_or(GrabStatus::ALREADY_GRABBED);

            if kb_status == GrabStatus::SUCCESS && ptr_status == GrabStatus::SUCCESS {
                return;
            }
        }

        thread::sleep(Duration::from_millis(50));
        if i % 10 == 0 {
            println!("Attempting to grab input...");
        }
    }

    eprintln!("CRITICAL: Failed to grab inputs!");
}
