use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use rand::seq::SliceRandom;
use rusttype::Font;
use users::get_current_username;
use x11rb::connection::Connection;

use crate::auth;
use crate::display::{DisplaySetup, Monitor};
use crate::input::keycode_to_char;
use crate::renderer::Renderer;
use crate::theme::{Theme, FUNNY_PHRASES};

// Maximum usable text width inside the login card (450px box minus 20px margin each side).
const CARD_CONTENT_W: i32 = 410;

pub struct ArcticLock {
    display: DisplaySetup,
    renderer: Renderer,
    theme: Theme,

    user: String,
    password: String,
    status_msg: String,
    status_color: u32,
    funny_phrase: String,

    shake_intensity: i32,
    blink_timer: u32,
    shift_pressed: bool,
    loading: bool,
}

impl ArcticLock {
    pub fn new(
        font_path: &str,
        bg_path: Option<String>,
        theme: Theme,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display = DisplaySetup::new()?;

        let font_data = std::fs::read(font_path)?;
        let font = Font::try_from_vec(font_data).ok_or("Failed to load font")?;

        let background = bg_path
            .filter(|p| Path::new(p).exists())
            .and_then(|p| {
                println!("Loading background: {}", p);
                image::open(&p).ok().map(|img| {
                    img.resize_exact(
                        display.width as u32,
                        display.height as u32,
                        image::imageops::FilterType::Lanczos3,
                    )
                })
            });

        let renderer = Renderer::new(font, display.width, display.height, background, theme.base);

        let user = get_current_username()
            .map(|u| u.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".into());

        Ok(Self {
            display,
            renderer,
            status_color: theme.subtext,
            theme,
            user,
            password: String::new(),
            status_msg: "Enter Password".to_string(),
            funny_phrase: String::new(),
            shake_intensity: 0,
            blink_timer: 0,
            shift_pressed: false,
            loading: false,
        })
    }

    pub fn run(&mut self) {
        loop {
            let start = Instant::now();

            while let Some(event) = self.display.conn.poll_for_event().unwrap() {
                match event {
                    x11rb::protocol::Event::KeyPress(ev) => self.handle_key(ev.detail, true),
                    x11rb::protocol::Event::KeyRelease(ev) => self.handle_key(ev.detail, false),
                    _ => {}
                }
            }

            self.render();

            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(16) {
                thread::sleep(Duration::from_millis(16) - elapsed);
            }
        }
    }

    fn render(&mut self) {
        self.renderer.clear();

        let mut shake_offset = 0;
        if self.shake_intensity > 0 {
            self.shake_intensity -= 1;
            shake_offset = (self.shake_intensity as f32 * 0.5).sin() as i32 * 10;
        }
        self.blink_timer = self.blink_timer.wrapping_add(1);

        let monitors: Vec<Monitor> = self.display.monitors.clone();
        for m in monitors {
            self.draw_ui_for_monitor(m, shake_offset);
        }

        self.renderer
            .present(&self.display.conn, self.display.window, self.display.gcontext);
    }

    fn draw_ui_for_monitor(&mut self, m: Monitor, shake_offset: i32) {
        let cx = m.x as i32 + m.w as i32 / 2 + shake_offset;
        let cy = m.y as i32 + m.h as i32 / 2;

        self.draw_clock(cx, cy);
        self.draw_login_box(cx, cy);
    }

    fn draw_clock(&mut self, cx: i32, cy: i32) {
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%A, %B %d").to_string();

        let time_w = self.renderer.measure_text(&time_str, 120.0);
        self.renderer
            .draw_text(&time_str, cx - time_w / 2, cy - 300, 120.0, self.theme.text);

        let date_w = self.renderer.measure_text(&date_str, 40.0);
        self.renderer
            .draw_text(&date_str, cx - date_w / 2, cy - 190, 40.0, self.theme.sky);
    }

    fn draw_login_box(&mut self, cx: i32, cy: i32) {
        const BOX_W: i32 = 450;
        const BOX_H: i32 = 250;

        let box_x = cx - BOX_W / 2;
        let box_y = cy - BOX_H / 2;

        self.renderer
            .draw_rect(box_x, box_y, BOX_W as u32, BOX_H as u32, self.theme.mantle);
        self.renderer
            .draw_rect(box_x, box_y, BOX_W as u32, 2, self.theme.teal);
        self.renderer
            .draw_rect(box_x, box_y + BOX_H - 2, BOX_W as u32, 2, self.theme.teal);

        let user_str = format!("User: {}", self.user);
        self.draw_text_fit(&user_str, cx, cy - 60, 32.0, self.theme.text);

        self.draw_password_field(cx, cy);
        self.draw_status(cx, cy);
    }

    fn draw_password_field(&mut self, cx: i32, cy: i32) {
        // Hidden while auth is running — the spinner communicates state instead.
        if self.loading {
            return;
        }

        if self.password.is_empty() {
            let placeholder = "Start Typing...";
            let ph_w = self.renderer.measure_text(placeholder, 32.0);
            self.renderer
                .draw_text(placeholder, cx - ph_w / 2, cy + 10, 32.0, self.theme.subtext);
        } else {
            // Cap displayed dots so they never overflow the card.
            let display_len = self.password.len().min(18);
            let pass_str = "• ".repeat(display_len);

            let pass_w = self.renderer.measure_text(&pass_str, 32.0);
            let pass_x = cx - pass_w / 2;
            self.renderer
                .draw_text(&pass_str, pass_x, cy + 10, 32.0, self.theme.sky);

            if (self.blink_timer / 15) % 2 == 0 {
                self.renderer
                    .draw_rect(pass_x + pass_w + 2, cy + 15, 2, 25, self.theme.teal);
            }
        }
    }

    fn draw_status(&mut self, cx: i32, cy: i32) {
        if self.loading {
            self.draw_spinner(cx, cy);
            return;
        }

        let status_msg = self.status_msg.clone();
        let status_color = self.status_color;
        self.draw_text_fit(&status_msg, cx, cy + 60, 20.0, status_color);

        if !self.funny_phrase.is_empty() {
            let phrase = self.funny_phrase.clone();
            self.draw_text_fit(&phrase, cx, cy + 90, 20.0, self.theme.peach);
        }
    }

    fn draw_spinner(&mut self, cx: i32, cy: i32) {
        const DOTS: u32 = 3;
        const SPACING: i32 = 26;
        const SIZE_ACTIVE: u32 = 12;
        const SIZE_IDLE: u32 = 8;

        // One dot advances every 10 frames (~166 ms at 60 fps).
        let active = (self.blink_timer / 10) % DOTS;
        let baseline_y = cy + 60;

        for i in 0..DOTS {
            let is_active = i == active;
            let size = if is_active { SIZE_ACTIVE } else { SIZE_IDLE };
            let color = if is_active { self.theme.teal } else { self.theme.subtext };

            let dot_cx = cx + (i as i32 - (DOTS as i32 - 1) / 2) * SPACING;
            let dot_x = dot_cx - size as i32 / 2;
            let dot_y = baseline_y - size as i32 / 2;

            self.renderer.draw_rect(dot_x, dot_y, size, size, color);
        }
    }

    /// Draw `text` centered at `cx`, scaling it down proportionally if it exceeds `CARD_CONTENT_W`.
    fn draw_text_fit(&mut self, text: &str, cx: i32, y: i32, ideal_scale: f32, color: u32) {
        let w = self.renderer.measure_text(text, ideal_scale);
        let scale = if w > CARD_CONTENT_W {
            ideal_scale * CARD_CONTENT_W as f32 / w as f32
        } else {
            ideal_scale
        };
        let w = self.renderer.measure_text(text, scale);
        self.renderer.draw_text(text, cx - w / 2, y, scale, color);
    }

    fn handle_key(&mut self, keycode: u8, pressed: bool) {
        if keycode == 50 || keycode == 62 {
            self.shift_pressed = pressed;
            return;
        }
        if !pressed || self.loading {
            return;
        }

        self.status_msg = "Authenticating...".to_string();
        self.status_color = self.theme.subtext;
        self.funny_phrase.clear();

        match keycode {
            36 => self.try_authenticate(),
            22 => {
                self.password.pop();
            }
            9 => self.password.clear(),
            _ => {
                if let Some(ch) = keycode_to_char(keycode, self.shift_pressed) {
                    self.password.push(ch);
                }
            }
        }
    }

    fn try_authenticate(&mut self) {
        let user = self.user.clone();
        let password = self.password.clone();
        self.password.clear();

        let (tx, rx) = mpsc::channel::<bool>();
        thread::spawn(move || {
            let _ = tx.send(auth::authenticate(&user, &password));
        });

        self.loading = true;

        loop {
            // Drain X11 events — this also flushes the put_image send queue.
            while self.display.conn.poll_for_event().ok().flatten().is_some() {}

            self.render();

            match rx.try_recv() {
                Ok(true) => std::process::exit(0),
                Ok(false) => {
                    self.loading = false;
                    self.status_msg = "Access Denied".to_string();
                    self.status_color = self.theme.red;
                    self.shake_intensity = 20;
                    self.funny_phrase = FUNNY_PHRASES
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string();
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(16));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.loading = false;
                    return;
                }
            }
        }
    }
}
