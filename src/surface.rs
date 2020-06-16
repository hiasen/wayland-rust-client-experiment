use std::cell::RefCell;
use std::rc::Rc;

use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm},
    Filter, Main,
};

use super::painter;
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_wm_base};

pub fn setup(
    compositor: &Main<WlCompositor>,
    xdg_wm_base: &Main<xdg_wm_base::XdgWmBase>,
    shm: &Main<WlShm>,
) {
    let surface = compositor.create_surface();
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    let toplevel = xdg_surface.get_toplevel();
    toplevel.set_title(String::from("Example client"));
    surface.commit();

    let painter = Rc::new(RefCell::new(painter::Painter::new(&shm)));

    xdg_surface.quick_assign({

        let surface = surface.clone();
        let painter = painter.clone();

        move |xdg_surface, event, _| match event {
            xdg_surface::Event::Configure { serial } => {
                xdg_surface.ack_configure(serial);
                let buffer = painter.borrow().draw()
                    .expect("Failed to draw first frame");
                surface.attach(Some(&buffer), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();

                // Remove this closure after drawing once
                xdg_surface.quick_assign(|_,_,_| ());
            }
            _ => (),
        }
    });

    surface.frame().assign(Filter::new({
        let surface = surface.clone();
        let painter = painter.clone();

        use wayland_client::protocol::wl_callback::Event::Done;
        move |event, filter, _| match event {
            (_, Done { callback_data: time }) => {
                surface.frame().assign(filter.clone());
                let mut painter = painter.borrow_mut();
                painter.update_time(time);
                let buffer = painter.draw()
                    .expect("Failed to draw frame");
                surface.attach(Some(&buffer), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();
            }
            _ => (),
        }
    }));
}
