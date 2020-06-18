use std::rc::Rc;
use std::cell::RefCell;

use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm, wl_surface::WlSurface, wl_callback},
    Filter, Main,
};

use crate::painter::Painter;
use crate::buffer::Buffer;
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_wm_base};
use wayland_protocols::xdg_shell::client::xdg_toplevel::Event as ToplevelEvent;

const WIDTH: usize = 600;
const HEIGHT: usize = 400;

pub struct State {
    surface: Main<WlSurface>,
    width: usize,
    height: usize,
    asked_to_close: bool,
    has_drawn: bool,
    shm: Main<WlShm>,
    painter: Painter,
}


impl State {

    fn new(surface: &Main<WlSurface>, shm: &Main<WlShm>) -> Self {
        Self {
            surface: surface.clone(),
            width: WIDTH,
            height: HEIGHT,
            asked_to_close: false,
            has_drawn: false,
            shm: shm.clone(),
            painter: Painter::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.asked_to_close
    }

    fn set_geometry(&mut self, width: usize, height: usize) {
        if width == 0 || height == 0 {
            self.width = WIDTH;
            self.height = HEIGHT;
        } else {
            self.width = width;
            self.height = height;
        };
    }

    fn draw(&self) {
        let mut buffer = Buffer::new(&self.shm, self.width, self.height)
            .expect("Failed to create buffer");
        self.painter.draw(&mut buffer);
        self.surface.attach(Some(buffer.wl_buffer()), 0, 0);
        self.surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
        self.surface.commit();
    }

    fn handle_toplevel(&mut self, event: ToplevelEvent) {
        use ToplevelEvent::*;
        match event {
            Configure { width, height, .. } => {
                self.set_geometry(width as usize, height as usize);
            }
            Close {} => {
                self.asked_to_close = true;
            }
            _ => ()
        }
    }

    fn handle_xdg_surface(
        &mut self,
        xdg_surface: xdg_surface::XdgSurface,
        event: xdg_surface::Event
    ) {
        match event {
            xdg_surface::Event::Configure { serial } => {
                xdg_surface.ack_configure(serial);
                if !self.has_drawn {
                    self.draw();
                    self.has_drawn = true;
                }
            }
            _ => (),
        }
    }

    fn handle_frame_callback(
        &mut self,
        filter: Filter<(Main<wl_callback::WlCallback>, wl_callback::Event)>,
        time: u32,
    ) {
        self.surface.frame().assign(filter);
        self.painter.update_time(time);
        self.draw();
    }
}


pub fn setup(
    compositor: &Main<WlCompositor>,
    xdg_wm_base: &Main<xdg_wm_base::XdgWmBase>,
    shm: &Main<WlShm>,
) -> Rc<RefCell<State>> {
    let surface = compositor.create_surface();
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    let toplevel = xdg_surface.get_toplevel();
    toplevel.set_title(String::from("Example client"));
    surface.commit();

    let state = Rc::new(RefCell::new(State::new(&surface, &shm)));
    
    toplevel.quick_assign({
        let state = state.clone();
        move |_, event, _| {
            state.borrow_mut().handle_toplevel(event);
        }
    });

    xdg_surface.quick_assign({
        let state = state.clone();
        move |xdg_surface, event, _| {
            state.borrow_mut()
                .handle_xdg_surface(xdg_surface.detach(), event);
        }
    });

    surface.frame().assign(Filter::new({
        let state = state.clone();
        use wl_callback::Event::Done;
        move |event, filter, _| match event {
            (_, Done { callback_data: time }) => {
                state.borrow_mut()
                    .handle_frame_callback(filter.clone(), time);
            }
            _ => (),
        }
    }));
    state
}
