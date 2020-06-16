use wayland_client::{
    Main,
    protocol::{
        wl_seat::{
            WlSeat,
            Capability,
            Event::{
                Capabilities,
                Name,
            },
        },
    },
};

mod pointer;
mod keyboard;

pub fn handle(seat: &Main<WlSeat>) {
    let mut pointer_handler = pointer::Handler::new(&seat);
    let mut keyboard_handler = keyboard::Handler::new(&seat);
    seat.quick_assign(move |_seat, event, _| {
        match event {
            Capabilities { capabilities: cap } => {
                let have_pointer = cap.contains(Capability::Pointer);
                pointer_handler.status_update(have_pointer);
                let have_keyboard = cap.contains(Capability::Keyboard);
                keyboard_handler.status_update(have_keyboard);
            },
            Name { name } => eprintln!("seat name: {}", name),
            _ => ()
        };
    });
}
