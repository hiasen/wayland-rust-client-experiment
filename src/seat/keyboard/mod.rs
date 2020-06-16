use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::rc::Rc;

use wayland_client::{
	protocol::{
		wl_keyboard::{Event::*, KeymapFormat, WlKeyboard},
		wl_seat::WlSeat,
	},
	Main,
};

mod missing_xkb_functions;
use missing_xkb_functions::keymap_from_buffer;

pub struct Handler {
	current_keyboard: Option<WlKeyboard>,
	seat: Main<WlSeat>,
	xkb_context: Rc<xkb::Context>,
}

impl Handler {
	pub fn new(seat: &Main<WlSeat>) -> Self {
		Self {
			current_keyboard: None,
			seat: seat.clone(),
			xkb_context: Default::default(),
		}
	}

	pub fn status_update(&mut self, seat_has_keyboard: bool) {
		let keyboard_is_configured = self.current_keyboard.is_some();
		if seat_has_keyboard && !keyboard_is_configured {
			self.setup_keyboard();
		} else if !seat_has_keyboard && keyboard_is_configured {
			self.release_keyboard()
		}
	}

	fn setup_keyboard(&mut self) {
		let keyboard = self.seat.get_keyboard();
		let xkb_context = self.xkb_context.clone();
		let mut keymap = None as Option<xkb::Keymap>;
		let mut state = None as Option<xkb::State>;

		keyboard.quick_assign(move |_, event, _| {
			match event {
				Keymap { format, fd, size } => {
					assert_eq!(format, KeymapFormat::XkbV1);
					let file = unsafe { File::from_raw_fd(fd) };
					// The file contains a cstring, but we will rather treat it as a buffer with known size.
					// So strip away the trailing null byte.
					let size = (size - 1) as usize;
					let map = unsafe { memmap2::MmapOptions::new().len(size).map(&file) }
					.expect("Failed to mmap keymap fd");

					let new_keymap = keymap_from_buffer(&xkb_context, &map)
					.expect("Failed to create keymap from buffer");
					let new_state = new_keymap.state();
					keymap = Some(new_keymap);
					state = Some(new_state);
				}
				Enter {
					serial: _,
					surface: _,
					keys,
				} => {
					let state = state.as_ref().unwrap();
					eprintln!("keyboard enter keys pressed are: ");

					let keys = keys.as_slice();
					// Assume keys are already aligned
					let (_, keys, _) = unsafe { keys.align_to::<u32>() };
					for key in keys.iter() {
						let key = state.key(*key + 8);

						if let Some(sym) = key.sym() {
							let sym_num: u32 = sym.into();
							eprint!("sym: {} ({}), ", sym.to_string(), sym_num);
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
					let state = state.as_ref().unwrap();
					let key = state.key(key + 8);
					eprint!("key {:?}: ", key_state);
					if let Some(sym) = key.sym() {
						eprint!("sym: {} ", sym.to_string());
					}
					if let Some(utf8_string) = key.utf8() {
						eprint!("utf8: {}", utf8_string);
					}
					eprintln!();
				}
				Leave {
					serial: _,
					surface: _,
				} => {
					eprintln!("keyboard leave");
				}
				Modifiers {
					mods_depressed,
					mods_latched,
					mods_locked,
					group,
					..
				} => {
					let state = state.as_mut().unwrap();
					state
					.update()
					.mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
				}
				RepeatInfo { .. } => (),
				_ => (),
			}
		});
		self.current_keyboard = Some(keyboard.detach());
	}

	fn release_keyboard(&mut self) {
		self.current_keyboard.take().unwrap().release();
	}
}
