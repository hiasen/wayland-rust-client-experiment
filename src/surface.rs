use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm},
    Filter, Main,
};

use crate::painter::Painter;
use crate::buffer::Buffer;
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_wm_base};

const WIDTH: usize = 600;
const HEIGHT: usize = 400;

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


    xdg_surface.quick_assign({

        let surface = surface.clone();
        let mut buffer = Buffer::new(&shm, WIDTH, HEIGHT)
            .expect("Failed to create buffer");

        move |xdg_surface, event, _| match event {
            xdg_surface::Event::Configure { serial } => {
                xdg_surface.ack_configure(serial);
                Painter::draw_once(&mut buffer);
                surface.attach(Some(buffer.wl_buffer()), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();

                // Remove this closure after drawing once
                xdg_surface.quick_assign(|_,_,_| ());
            }
            _ => (),
        }
    });


    surface.frame().assign(Filter::new({
        let shm = shm.clone();
        let surface = surface.clone();
        let mut painter = Painter::new();

        use wayland_client::protocol::wl_callback::Event::Done;
        move |event, filter, _| match event {
            (_, Done { callback_data: time }) => {
                surface.frame().assign(filter.clone());
                painter.update_time(time);
                let mut buffer = Buffer::new(&shm, WIDTH, HEIGHT)
                    .expect("Failed to create buffer");
                painter.draw(&mut buffer);
                surface.attach(Some(buffer.wl_buffer()), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();
            }
            _ => (),
        }
    }));
}
