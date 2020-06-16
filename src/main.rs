use std::error::Error;

use wayland_client::{
	Display,
	GlobalManager,
	protocol::{
		wl_compositor::WlCompositor,
		wl_shm::WlShm,
		wl_seat::WlSeat,
	},
};

use wayland_protocols::xdg_shell::client::{
	xdg_wm_base,
};

mod painter;
mod debug_callbacks;
mod shared_memory;
mod seat;
mod surface;

fn main() -> Result<(), Box<dyn Error>>{
	let display = Display::connect_to_env()?;
	let mut event_queue = display.create_event_queue();
	let token = event_queue.token();
	let attached = display.attach(token);
	let global = GlobalManager::new_with_cb(&attached, debug_callbacks::print_global_event);
	event_queue.sync_roundtrip(&mut (), |_,_,_| { unreachable!(); })?;

	// Globals
	let compositor = global.instantiate_exact::<WlCompositor>(4)?;
	let xdg_wm_base = global.instantiate_exact::<xdg_wm_base::XdgWmBase>(1)?;
	let shm = global.instantiate_exact::<WlShm>(1)?;
	let seat = global.instantiate_exact::<WlSeat>(5)?;

	xdg_wm_base.quick_assign(|obj, event, _| {
		if let xdg_wm_base::Event::Ping { serial } = event {
			obj.pong(serial);
		}
	});

	surface::setup(
		&compositor,
		&xdg_wm_base,
		&shm,
	);
	seat::handle(&seat);

	loop {
		event_queue.dispatch(&mut (), |_, _, _| {})?;
	}
}
