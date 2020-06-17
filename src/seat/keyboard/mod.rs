use std::fs::File;
use std::os::unix::io::FromRawFd;

use wayland_client::{
    protocol::wl_keyboard::{Event::*, KeymapFormat, WlKeyboard},
    Main,
};

mod missing_xkb_functions;
use missing_xkb_functions::keymap_from_buffer;

pub fn handle(keyboard: &Main<WlKeyboard>) {
    let context = xkb::Context::default();
    keyboard.quick_assign(move |keyboard, event, _| match event {
        Keymap { format, fd, size } => {
            assert_eq!(format, KeymapFormat::XkbV1);
            let mut file = unsafe { File::from_raw_fd(fd) };
            let state = get_state(&context, &mut file, size as usize)
                .expect("Failed to create first state");
            handle_after_first_keymap_event(keyboard, state, context.clone());
        }
        _ => (),
    });
}

fn get_state(
    xkb_context: &xkb::Context,
    file: &mut File,
    size: usize,
) -> Result<xkb::State, Box<dyn std::error::Error>> {
    let buffer = unsafe { memmap2::MmapOptions::new().len(size).map(&file) }?;
    // Treating the buffer as a slice of chars instead of a CString
    // by stripping the trailing null byte.
    let keymap = keymap_from_buffer(&xkb_context, &buffer[..size-1])?;
    let state = keymap.state();
    Ok(state)
}

fn handle_after_first_keymap_event(
    keyboard: Main<WlKeyboard>,
    state: xkb::State,
    context: xkb::Context,
) {
    let mut state = state;

    keyboard.quick_assign(move |_, event, _| match event {
        Keymap { format, fd, size } => {
            assert_eq!(format, KeymapFormat::XkbV1);
            let mut file = unsafe { File::from_raw_fd(fd) };
            match get_state(&context, &mut file, size as usize) {
                Ok(new_state) => {
                    state = new_state;
                }
                Err(error) => {
                    eprintln!("Failed to set new state after keymap event: {}", error);
                }
            }
        }
        Enter {
            serial: _,
            surface: _,
            keys,
        } => {
            eprintln!("keyboard enter keys pressed are: ");

            // Assume keys are already aligned
            let keys = unsafe { keys.as_slice().align_to::<u32>() }.1
                .iter()
                .map(|key| state.key(*key + 8));
            for key in keys {
                if let Some(sym) = key.sym() {
                    eprint!("sym: {} ({}), ", sym.to_string(), sym.0);
                } else {
                    eprint!("sym: Unknown ");
                }

                if let Some(utf8_string) = key.utf8() {
                    eprint!("utf8: {}", utf8_string);
                }

                eprintln!();
            }
        }
        Key {
            key,
            state: key_state,
            ..
        } => {
            eprint!("key {:?}: ", key_state);

            let key = state.key(key + 8);

            if let Some(sym) = key.sym() {
                eprint!("sym: {} ", sym.to_string());
            }

            if let Some(utf8_string) = key.utf8() {
                eprint!("utf8: {}", utf8_string);
            }

            eprintln!();
        }
        Leave { .. } => {
            eprintln!("keyboard leave");
        }
        Modifiers {
            mods_depressed,
            mods_latched,
            mods_locked,
            group,
            ..
        } => {
            state
                .update()
                .mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
        }
        RepeatInfo { .. } => (),
        _ => (),
    });
}
