mod app;
mod auth;
mod display;
mod input;
mod renderer;
mod theme;

use std::env;

use app::ArcticLock;
use theme::Theme;

fn main() {
    let (font_path, bg_path, theme) = parse_args();

    match ArcticLock::new(&font_path, bg_path, theme) {
        Ok(mut lock) => lock.run(),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn parse_args() -> (String, Option<String>, Theme) {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut font: Option<String> = None;
    let mut bg: Option<String> = None;
    let mut theme = Theme::mocha();

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--theme" {
            i += 1;
            if let Some(name) = args.get(i) {
                theme = Theme::from_name(name).unwrap_or_else(|| {
                    eprintln!(
                        "Unknown theme '{}'. Valid options: mocha, macchiato, frappe, latte. Using mocha.",
                        name
                    );
                    Theme::mocha()
                });
            } else {
                eprintln!("--theme requires an argument.");
                print_usage();
                std::process::exit(1);
            }
        } else if font.is_none() {
            font = Some(args[i].clone());
        } else if bg.is_none() {
            bg = Some(args[i].clone());
        }
        i += 1;
    }

    let font = font.unwrap_or_else(|| {
        print_usage();
        std::process::exit(1);
    });

    (font, bg, theme)
}

fn print_usage() {
    eprintln!(
        "Usage: arctic-lock <font.ttf> [background.png] [--theme mocha|macchiato|frappe|latte]"
    );
}
