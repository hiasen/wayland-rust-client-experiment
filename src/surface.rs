use std::rc::Rc;
use std::cell::RefCell;

use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm},
    Filter, Main,
};

use crate::painter::Painter;
use crate::buffer::Buffer;
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_wm_base};

const WIDTH: usize = 600;
const HEIGHT: usize = 400;

struct Geometry {
    width: usize,
    height: usize,
}
impl Default for Geometry {
    fn default() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

pub fn setup(
    compositor: &Main<WlCompositor>,
    xdg_wm_base: &Main<xdg_wm_base::XdgWmBase>,
    shm: &Main<WlShm>,
) -> Rc<RefCell<bool>> {
    let surface = compositor.create_surface();
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    let toplevel = xdg_surface.get_toplevel();
    toplevel.set_title(String::from("Example client"));
    surface.commit();

    let is_closed = Rc::new(RefCell::new(false));
    let geometry = Rc::new(RefCell::new(Geometry::default()));

    toplevel.quick_assign({
        let is_closed = is_closed.clone();
        let geometry = geometry.clone();
        use wayland_protocols::xdg_shell::client::xdg_toplevel::Event::*;
        move |_, event, _| match event {
            Configure { width, height, .. } => {
                geometry.replace(
                    if width == 0 || height == 0 {
                        Default::default()
                    } else {
                        Geometry {width: width as usize, height: height as usize}
                    }
                );
            }
            Close {} => {
                is_closed.replace(true);
            }
            _ => ()

        }
    });

    xdg_surface.quick_assign({

        let surface = surface.clone();
        let shm = shm.clone();
        let mut has_drawn = false;

        move |xdg_surface, event, _| match event {
            xdg_surface::Event::Configure { serial } => {
                xdg_surface.ack_configure(serial);
                if !has_drawn {
                    let mut buffer = Buffer::new(&shm, WIDTH, HEIGHT)
                        .expect("Failed to create buffer");
                    Painter::draw_once(&mut buffer);
                    surface.attach(Some(buffer.wl_buffer()), 0, 0);
                    surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                    surface.commit();
                    has_drawn = true;
                }
            }
            _ => (),
        }
    });


    surface.frame().assign(Filter::new({
        let shm = shm.clone();
        let surface = surface.clone();
        let mut painter = Painter::new();
        let geometry = geometry.clone();

        use wayland_client::protocol::wl_callback::Event::Done;
        move |event, filter, _| match event {
            (_, Done { callback_data: time }) => {
                surface.frame().assign(filter.clone());
                let geometry = geometry.borrow();
                painter.update_time(time);
                let mut buffer = Buffer::new(&shm, geometry.width, geometry.height)
                    .expect("Failed to create buffer");
                painter.draw(&mut buffer);
                surface.attach(Some(buffer.wl_buffer()), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();
            }
            _ => (),
        }
    }));
    is_closed
}
