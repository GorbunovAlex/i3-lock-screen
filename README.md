# Arctic Lock ❄️

⚠️ **Educational Project - Use With Caution** ⚠️

A high-performance, aesthetically pleasing X11 screen locker written in Rust. This is a learning project exploring X11 screen locking mechanisms. While functional, this is **NOT recommended for security-critical environments**. For production use, consider mature alternatives like `i3lock`, `xsecurelock`, or `swaylock`.

Arctic Lock renders a synchronized lock screen across all connected monitors, supports custom backgrounds, Catppuccin themes, and shows random dev humor on failed login attempts.

---

## ⚠️ SECURITY NOTICE

**READ THIS BEFORE INSTALLING**

### Critical Security Information

- **SUID Root Binary**: This program requires root privileges to access PAM authentication. Any bugs in the code could potentially lead to privilege escalation.
- **Educational Purpose**: This is a learning project and proof of concept, not enterprise security software.
- **Use Only If**: You understand the security implications, have reviewed the source code, and accept the risks.
- **Not Recommended For**: Production environments, shared systems, multi-user environments, or high-security scenarios.

### Known Limitations

- ⚠️ Password stored in plain String (not cleared from memory)
- ⚠️ If input grab fails, program continues running (may appear locked when it's not)
- ⚠️ No rate limiting on authentication attempts
- ⚠️ No account lockout after failed attempts
- ⚠️ PAM initialization errors will panic (could unlock screen)
- ⚠️ No privilege dropping after initialization

**Recommendation**: For production use or security-critical environments, use the mature alternatives listed below.

### Mature Alternatives

- **i3lock** - Minimal, proven screen locker
- **xsecurelock** - Extensive security features
- **swaylock** - For Wayland compositors
- **slock** - Suckless simple locker

### Reporting Security Issues

If you discover a security vulnerability, please report it privately before public disclosure. Do not open public GitHub issues for security problems.

---

## ✨ Features

- **Multi-Monitor Support**: Automatically detects all connected screens via RandR and renders the lock UI centered on each one.
- **Catppuccin Themes**: Four flavors — Mocha, Macchiato, Frappé, Latte — selectable at launch via `--theme`.
- **Security**:
  - Bypasses window managers using `override_redirect`
  - Keyboard and pointer grab to prevent window switching
  - PAM authentication (Pluggable Authentication Modules)
- **Animations**: Shake-on-error and blinking cursor.
- **Typography**: Clean TTF rendering via `rusttype`.
- **Custom Wallpaper**: Loads and scales any image to fit the screen.
- **Dev Humor**: 44 random phrases shown on failed login attempts.

---

## 🎨 Themes

Arctic Lock ships with all four [Catppuccin](https://github.com/catppuccin/catppuccin) flavors:

| Name | Style | `--theme` value |
|------|-------|-----------------|
| Mocha | Dark (darkest) | `mocha` |
| Macchiato | Dark | `macchiato` |
| Frappé | Dark (medium) | `frappe` |
| Latte | Light | `latte` |

The default theme is **Mocha**. Pass `--theme <name>` anywhere in the argument list to change it.

---

## 🛠️ Prerequisites

### Debian / Ubuntu

```bash
sudo apt update
sudo apt install cargo libpam0g-dev libx11-dev libxrandr-dev
```

### Arch Linux

```bash
sudo pacman -S rust cargo pam libx11 libxrandr
```

### Fedora / RHEL

```bash
sudo dnf install cargo pam-devel libX11-devel libXrandr-devel
```

---

## 📦 Installation

### 1. Clone the repository

```bash
git clone https://github.com/GorbunovAlex/arctic-lock-screen.git
cd arctic-lock-screen
```

### 2. Review the source code

**IMPORTANT**: Before building and installing any SUID binary, review the source code to ensure you understand and trust it.

### 3. Build the release binary

```bash
cargo build --release
```

### 4. Install system-wide

> **⚠️ CRITICAL SECURITY WARNING**: Arctic Lock requires root ownership and the setuid bit to access PAM. This means any vulnerability in the code could be exploited for privilege escalation. Only proceed if you understand and accept this risk.

```bash
sudo mv target/release/arctic-lock /usr/local/bin/
sudo chown root:root /usr/local/bin/arctic-lock
sudo chmod 4755 /usr/local/bin/arctic-lock
```

### 5. Verify installation

```bash
ls -l /usr/local/bin/arctic-lock
# Should show: -rwsr-xr-x 1 root root ...
```

---

## 🚀 Usage

```
arctic-lock <font.ttf> [background.png] [--theme mocha|macchiato|frappe|latte]
```

### Examples

```bash
# Default theme (Mocha), no background
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf

# With a background image
arctic-lock /usr/share/fonts/truetype/hack/Hack-Regular.ttf ~/Pictures/wallpaper.png

# Latte (light) theme
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf --theme latte

# Macchiato theme with a background
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf ~/Pictures/wallpaper.png --theme macchiato

# --theme can appear anywhere in the argument list
arctic-lock --theme frappe /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
```

### Finding fonts

```bash
fc-list | grep -i "dejavu\|hack\|liberation"
```

Common font paths:
- `/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf`
- `/usr/share/fonts/truetype/hack/Hack-Regular.ttf`
- `/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf`

### Binding to a keyboard shortcut

**i3 / i3-gaps:**

```
bindsym $mod+l exec --no-startup-id arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf --theme mocha
```

**bspwm:**

```
super + l
    arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf --theme mocha
```

---

## ⌨️ Controls

| Key | Action |
|-----|--------|
| Type | Append to password |
| `Enter` | Submit password |
| `Backspace` | Delete last character |
| `Escape` | Clear the entire field |

---

## 🏗️ Architecture

The project is split into focused modules:

```
src/
  main.rs       — argument parsing, entry point
  app.rs        — ArcticLock struct: event loop, render dispatch, key handling
  display.rs    — X11 connection, window creation, monitor detection (RandR), input grab
  renderer.rs   — pixel buffer, draw_rect / draw_text / measure_text / present
  auth.rs       — PAM authentication
  input.rs      — keycode → char mapping
  theme.rs      — Theme struct with all four Catppuccin flavors + phrase list
```

**Rendering pipeline:**
1. `clear()` — fill buffer with base color or blit background image
2. `draw_*()` — software-rendered text and rects with alpha blending
3. `present()` — chunked `XPutImage` to the X server (avoids request size limits on 4K+)

**Authentication flow:**
1. Enter key → show "Authenticating..." → render frame → PAM call (blocking)
2. Success → `exit(0)`
3. Failure → clear password, set error state, pick random phrase, trigger shake animation

---

## 🔧 Troubleshooting

### "Authentication Failed" with correct password

Permissions are wrong. Verify:

```bash
ls -l /usr/local/bin/arctic-lock
# Must show -rwsr-xr-x 1 root root ...
```

If the `s` is missing:

```bash
sudo chmod 4755 /usr/local/bin/arctic-lock
```

### "CRITICAL ERROR: Failed to grab inputs"

Another application holds a keyboard/pointer grab (games, VMs, screen recorders). Close it and try again. If the grab fails, the screen may appear locked but inputs are not captured — this is a security risk.

### Screen doesn't cover all monitors

RandR detection failed. Check with `xrandr --listmonitors` and ensure all monitors are active.

### PAM errors

Check `/etc/pam.d/login` exists and is correctly configured for your distribution.

---

## 📋 System Requirements

- **OS**: Linux with X11 (Wayland is not supported — use `swaylock`)
- **Display Server**: X.Org
- **PAM**: System PAM configured
- **Rust**: 1.70.0 or newer

### Tested On

- Arch Linux (X11)
- Ubuntu 22.04 LTS (X11)
- Debian 12 (X11)
- Fedora 38 (X11)

---

## 🗺️ Roadmap

- [x] Multi-monitor support
- [x] Custom background image
- [x] Catppuccin theme support (Mocha, Macchiato, Frappé, Latte)
- [x] Modular codebase
- [ ] Configuration file support
- [ ] Screen blanking after idle timeout
- [ ] Privilege dropping after PAM initialization
- [ ] Wayland support (via wlroots)
