//! The injection — types via `zwp_virtual_keyboard_v1` into the
//! FOCUSED window, whatever toolkit lives there (MatrixKit, WebKit,
//! greeter password field). The trick comes from wtype: we upload our
//! OWN keymap in which every character of our keys has its own
//! keycode — umlauts and capitals included, without any modifier
//! gymnastics. A key press is then exactly one press/release pair
//! on "its" code.

use std::io::{Seek, Write};
use std::os::fd::AsFd;
use std::sync::mpsc;
use std::time::Instant;

use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

/// What the surface sends to the injection.
#[derive(Debug, Clone)]
pub enum Command {
    /// A character of our keys (incl. umlauts, capitals, space).
    Char(char),
    /// A named XKB keysym: "Return", "BackSpace", "Tab", "Escape".
    Name(&'static str),
}

struct State {
    seat: Option<wl_seat::WlSeat>,
    manager: Option<ZwpVirtualKeyboardManagerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_seat" => {
                    state.seat = Some(registry.bind(name, version.min(7), qh, ()));
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    state.manager = Some(registry.bind(name, 1, qh, ()));
                }
                _ => {}
            }
        }
    }
}

wayland_client::delegate_noop!(State: ignore wl_seat::WlSeat);
wayland_client::delegate_noop!(State: ignore ZwpVirtualKeyboardManagerV1);
wayland_client::delegate_noop!(State: ignore ZwpVirtualKeyboardV1);

/// Keysym-Name eines Zeichens: Unicode-Schreibweise (libxkbcommon versteht
/// `U00E4` for ä), only the space gets its classic name.
fn keysym(glyph: char) -> String {
    if glyph == ' ' {
        String::from("space")
    } else {
        format!("U{:04X}", glyph as u32)
    }
}

/// The complete keymap: entry i → XKB keycode i+9 → wire code i+1
/// (Wayland subtracts 8). Types/compat come from the system XKB
/// data ("complete") that every compositor loads anyway.
///
/// `variante` stamps a dummy entry onto keycode 254 (F19/F20, never
/// sent): Smithay deduplicates keymaps via a SHA-256 of the
/// SERIALIZED content and seat-GLOBALLY — windows focused AFTER our
/// upload only know the real keyboard keymap and misread our codes
/// (user finding: "." typed "q"). Two alternating variants force a
/// fresh delivery to the currently focused window before every
/// key press.
fn keymap_text(entries: &[String], variante: bool) -> String {
    let mut s = String::from(
        "xkb_keymap {\nxkb_keycodes \"(unnamed)\" {\nminimum = 8;\nmaximum = 255;\n",
    );
    for i in 0..entries.len() {
        s.push_str(&format!("<K{}> = {};\n", i + 1, i + 9));
    }
    s.push_str("<KZZ> = 254;\n");
    s.push_str(
        "};\nxkb_types \"(unnamed)\" { include \"complete\" };\n\
         xkb_compatibility \"(unnamed)\" { include \"complete\" };\n\
         xkb_symbols \"(unnamed)\" {\n",
    );
    for (i, name) in entries.iter().enumerate() {
        s.push_str(&format!("key <K{}> {{[ {} ]}};\n", i + 1, name));
    }
    s.push_str(if variante { "key <KZZ> {[ F19 ]};\n" } else { "key <KZZ> {[ F20 ]};\n" });
    s.push_str("};\n};\n");
    s
}

/// The injection thread: own Wayland connection, own queue — the
/// iced surface sends commands over the channel. Returns `None` if
/// the compositor offers no virtual keyboard (the surface would show
/// keys, but typing into the void would be a lie — the caller may
/// report that).
pub fn start(glyph: Vec<char>) -> Option<mpsc::Sender<Command>> {
    let conn = Connection::connect_to_env().ok()?;
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    let display = conn.display();
    let _registry = display.get_registry(&qh, ());
    let mut state = State { seat: None, manager: None };
    queue.roundtrip(&mut state).ok()?;
    let seat = state.seat.clone()?;
    let manager = state.manager.clone()?;
    let vk = manager.create_virtual_keyboard(&seat, &qh, ());

    // Keymap: first the named special keys, then all characters.
    const SPECIAL: &[&str] = &["Return", "BackSpace", "Tab", "Escape"];
    let mut entries: Vec<String> = SPECIAL.iter().map(|s| s.to_string()).collect();
    let mut glyph = glyph;
    glyph.sort_unstable();
    glyph.dedup();
    for z in &glyph {
        entries.push(keysym(*z));
    }
    // Prepare both variants as file descriptors (see keymap_text:
    // alternating uploads force the delivery).
    let run = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    let mut files = Vec::new();
    for variante in [true, false] {
        let text = keymap_text(&entries, variante);
        let path = std::path::PathBuf::from(&run).join(format!(
            "matrix-keyboard-keymap-{}-{}",
            std::process::id(),
            variante
        ));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .ok()?;
        file.write_all(text.as_bytes()).ok()?;
        file.rewind().ok()?;
        let _ = std::fs::remove_file(&path);
        files.push((file, text.len() as u32));
    }
    vk.keymap(1, files[0].0.as_fd(), files[0].1);
    // A neutral modifier state is part of protocol politeness.
    vk.modifiers(0, 0, 0, 0);
    queue.roundtrip(&mut state).ok()?;

    // Draht-Code eines Befehls in UNSERER Keymap.
    let code_from = move |command: &Command| -> Option<u32> {
        let name = match command {
            Command::Name(n) => (*n).to_string(),
            Command::Char(z) => keysym(*z),
        };
        entries.iter().position(|e| *e == name).map(|i| i as u32 + 1)
    };

    let (sender, receiver) = mpsc::channel::<Command>();
    std::thread::spawn(move || {
        let start = Instant::now();
        let mut next = 1usize; // Start-Upload war dateien[0]
        for command in receiver {
            let Some(code) = code_from(&command) else { continue };
            // Before EVERY key press the other keymap variant: fresh SHA →
            // Smithay MUST deliver it to the focused window before the key
            // arrives (protocol order).
            let (file, length) = &files[next];
            vk.keymap(1, file.as_fd(), *length);
            next = 1 - next;
            let t = start.elapsed().as_millis() as u32;
            vk.key(t, code, 1);
            vk.key(t.saturating_add(8), code, 0);
            if queue.roundtrip(&mut state).is_err() {
                break; // Compositor weg — Faden endet leise.
            }
        }
    });
    Some(sender)
}

/// `--tipptest <text>`: type blindly into the focused window — the
/// proof that the injection carries, without anyone touching the
/// surface.
pub fn type_test(text: &str) {
    let Some(channel) = start(text.chars().collect()) else {
        eprintln!("tipptest: kein Virtual-Keyboard am Compositor");
        std::process::exit(1);
    };
    for z in text.chars() {
        let _ = channel.send(Command::Char(z));
    }
    // The thread drains the channel; wait briefly, then leave cleanly.
    std::thread::sleep(std::time::Duration::from_millis(400));
}
