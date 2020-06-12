use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use wayland_client::{
    Display,
    Filter,
    GlobalManager,
    Main,
    protocol::{
        wl_compositor::WlCompositor,
        wl_shm,
        wl_callback,
    },
};

use wayland_protocols::xdg_shell::client::{
    xdg_wm_base,
    xdg_surface,
};

mod painter;
mod debug_callbacks;
mod shared_memory;

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
    let shm = global.instantiate_exact::<wl_shm::WlShm>(1)?;
    
    xdg_wm_base.quick_assign(|obj, event, _| {
        if let xdg_wm_base::Event::Ping { serial } = event {
            obj.pong(serial);
        }
    });

    // Surface 
    let surface = compositor.create_surface();
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    let toplevel = xdg_surface.get_toplevel();
    toplevel.set_title(String::from("Example client"));
    surface.commit();
    
   
    let painter = Rc::new(RefCell::new(painter::Painter::new(shm.clone())));

    xdg_surface.quick_assign({
        let surface = surface.clone();
        let painter = painter.clone();
        move |xdg_surface, event, _| {
            if let xdg_surface::Event::Configure {serial} = event {
                xdg_surface.ack_configure(serial);
                let buffer = painter.borrow().draw().unwrap();
                surface.attach(Some(&buffer), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();
            }
        }
    });

    surface.frame().assign(Filter::<(Main<wl_callback::WlCallback>, wl_callback::Event)>::new({
        let surface = surface.clone();
        let painter = painter.clone();
        move |event, filter, _data| {
            if let (_, wl_callback::Event::Done {callback_data: time}) = event {
                surface.frame().assign(filter.clone());
                let mut painter = painter.borrow_mut();
                painter.update_time(time);
                let buffer = painter.draw().unwrap();
                surface.attach(Some(&buffer), 0, 0);
                surface.damage_buffer(0, 0, i32::MAX, i32::MAX);
                surface.commit();
            }
        }
    }));

    loop {
        event_queue.dispatch(&mut (), debug_callbacks::print_unfiltered_events)?;
    }
}
