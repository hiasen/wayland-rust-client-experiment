use std::cell::RefCell;
use std::rc::Rc;

use wayland_client::{
	Filter,
	Main,
	protocol::{
		wl_compositor::WlCompositor,
		wl_shm::WlShm,
	},
};

use wayland_protocols::xdg_shell::client::{
	xdg_wm_base,
	xdg_surface,
};
use super::painter;

pub fn setup(
	compositor: &Main<WlCompositor>,
	xdg_wm_base: &Main<xdg_wm_base::XdgWmBase>,
	shm: &Main<WlShm>) {
	let surface = compositor.create_surface();
	let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
	let toplevel = xdg_surface.get_toplevel();
	toplevel.set_title(String::from("Example client"));
	surface.commit();
		
		
	let painter = Rc::new(RefCell::new(painter::Painter::new(&shm)));
	xdg_surface.quick_assign({
		let surface = surface.clone();
		let painter = painter.clone();
		let mut is_first_frame = true;
		move |xdg_surface, event, _| {
			if let xdg_surface::Event::Configure {serial} = event {
				xdg_surface.ack_configure(serial);
				if is_first_frame {
					let buffer = {
						let painter = painter.borrow_mut();
						painter.draw()
					}.expect("Failed to draw first frame");
					surface.attach(Some(&buffer), 0, 0);
					surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
					surface.commit();
					is_first_frame = false;
				}
			}
		}
	});

	let filter = Filter::new({
		let surface = surface.clone();
		let painter = painter.clone();

		move |event, filter, _| {
			use wayland_client::protocol::wl_callback::Event::Done;
			if let (_, Done { callback_data: time }) = event {
				surface.frame().assign(filter.clone());
				let buffer = {
					let mut painter = painter.borrow_mut();
					painter.update_time(time);
					painter.draw()
				}.expect("Failed to draw frame");
				surface.attach(Some(&buffer), 0, 0);
				surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
				surface.commit();
			}
		}
	});
	surface.frame().assign(filter);        
}
