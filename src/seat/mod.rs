use wayland_client::{
    protocol::{
        wl_keyboard::WlKeyboard,
        wl_pointer::WlPointer,
        wl_seat::{
            Capability,
            Event::{Capabilities, Name},
            WlSeat,
        },
    },
    Main,
};

mod keyboard;
mod pointer;

pub fn handle(seat: &Main<WlSeat>) {
    let mut pointer = None as Option<WlPointer>;
    let mut keyboard = None as Option<WlKeyboard>;
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

                let seat_has_keyboard = cap.contains(Capability::Keyboard);
                let keyboard_created = keyboard.is_some();
                if seat_has_keyboard && !keyboard_created {
                    let new_keyboard = seat.get_keyboard();
                    keyboard::handle(&new_keyboard);
                    keyboard.replace(new_keyboard.detach());
                } else if !seat_has_keyboard && keyboard_created {
                    keyboard.take();
                }
            }
            Name { name } => eprintln!("seat name: {}", name),
            _ => (),
        };
    });
}
