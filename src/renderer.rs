use image::GenericImageView;
use rusttype::{Font, PositionedGlyph, Scale};
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub struct Renderer {
    pub buffer: Vec<u32>,
    font: Font<'static>,
    pub width: u16,
    pub height: u16,
    background: Option<image::DynamicImage>,
    base_color: u32,
}

impl Renderer {
    pub fn new(
        font: Font<'static>,
        width: u16,
        height: u16,
        background: Option<image::DynamicImage>,
        base_color: u32,
    ) -> Self {
        Self {
            buffer: vec![base_color; width as usize * height as usize],
            font,
            width,
            height,
            background,
            base_color,
        }
    }

    pub fn clear(&mut self) {
        match &self.background {
            Some(bg) => {
                for (i, pixel) in bg.pixels().enumerate() {
                    if i < self.buffer.len() {
                        let rgba = pixel.2;
                        self.buffer[i] = ((rgba[3] as u32) << 24)
                            | ((rgba[0] as u32) << 16)
                            | ((rgba[1] as u32) << 8)
                            | (rgba[2] as u32);
                    }
                }
            }
            None => self.buffer.fill(self.base_color),
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let alpha = (color >> 24) & 0xFF;
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px < 0 || px >= self.width as i32 || py < 0 || py >= self.height as i32 {
                    continue;
                }
                let idx = py as usize * self.width as usize + px as usize;
                if alpha == 255 {
                    self.buffer[idx] = color;
                } else {
                    self.buffer[idx] = blend(color, self.buffer[idx], alpha);
                }
            }
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, scale: f32, color: u32) -> u32 {
        let scale = Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(x as f32, y as f32 + v_metrics.ascent);
        let glyphs: Vec<PositionedGlyph> = self.font.layout(text, scale, offset).collect();
        let mut max_x = 0u32;

        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                max_x = max_x.max(bb.max.x as u32);
                let (w, h) = (self.width as i32, self.height as i32);
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;
                    if px < 0 || px >= w || py < 0 || py >= h {
                        return;
                    }
                    let idx = py as usize * self.width as usize + px as usize;
                    let alpha = (v * 255.0) as u32;
                    self.buffer[idx] = blend(color, self.buffer[idx], alpha);
                });
            }
        }

        max_x.saturating_sub(x as u32)
    }

    pub fn measure_text(&self, text: &str, scale: f32) -> i32 {
        let scale = Scale::uniform(scale);
        // Use advance position of the last glyph so trailing whitespace is included.
        // pixel_bounding_box() returns None for spaces, making the old approach return 0
        // for any string ending in a space (e.g. the "• " password mask).
        self.font
            .layout(text, scale, rusttype::point(0.0, 0.0))
            .last()
            .map(|g| (g.position().x + g.unpositioned().h_metrics().advance_width).ceil() as i32)
            .unwrap_or(0)
    }

    pub fn present(&self, conn: &RustConnection, window: Window, gcontext: Gcontext) {
        let data: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const u8,
                self.buffer.len() * 4,
            )
        };

        let stride = self.width as usize * 4;
        let rows_per_chunk = (64 * 1024 / stride).max(1);

        let mut y = 0u16;
        while y < self.height {
            let chunk_h = (self.height - y).min(rows_per_chunk as u16);
            let start = y as usize * stride;
            let end = start + chunk_h as usize * stride;

            if end > data.len() {
                break;
            }

            conn.put_image(
                ImageFormat::Z_PIXMAP,
                window,
                gcontext,
                self.width,
                chunk_h,
                0,
                y as i16,
                0,
                24,
                &data[start..end],
            )
            .unwrap();

            y += chunk_h;
        }
    }
}

#[inline]
fn blend(src: u32, dst: u32, alpha: u32) -> u32 {
    let inv_a = 255 - alpha;
    let r = (((src >> 16) & 0xFF) * alpha + ((dst >> 16) & 0xFF) * inv_a) / 255;
    let g = (((src >> 8) & 0xFF) * alpha + ((dst >> 8) & 0xFF) * inv_a) / 255;
    let b = ((src & 0xFF) * alpha + (dst & 0xFF) * inv_a) / 255;
    0xFF00_0000 | (r << 16) | (g << 8) | b
}
