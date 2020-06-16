use wayland_client::{
    protocol::{
        wl_seat::{
            Capability,
            Event::{Capabilities, Name},
            WlSeat,
        },
        wl_pointer::WlPointer,
    },
    Main,
};

mod keyboard;
mod pointer;

pub fn handle(seat: &Main<WlSeat>) {
    let mut pointer = None as Option<WlPointer>;
    let mut keyboard_handler = keyboard::Handler::new(&seat);
    seat.quick_assign(move |seat, event, _| {
        match event {
            Capabilities { capabilities: cap } => {

                let seat_has_pointer = cap.contains(Capability::Pointer);
                let pointer_created = pointer.is_some();
                if seat_has_pointer && !pointer_created {
                    let new_pointer = seat.get_pointer();
                    pointer::handle(&new_pointer);
                    pointer.replace(new_pointer.detach());
                } else if !seat_has_pointer && pointer_created {
                    pointer.take();
                }
                
                let have_keyboard = cap.contains(Capability::Keyboard);
                keyboard_handler.status_update(have_keyboard);
            }
            Name { name } => eprintln!("seat name: {}", name),
            _ => (),
        };
    });
}
