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

pub fn handle(seat: &Main<WlSeat>) {
    let mut pointer_handler = pointer::Handler::new(&seat);
    seat.quick_assign(move |_seat, event, _| {
        match event {
            Capabilities { capabilities: cap } => {
                let have_pointer = cap.contains(Capability::Pointer);
                pointer_handler.status_update(have_pointer);
            },
            Name { name } => eprintln!("seat name: {}", name),
            _ => ()
        };
    });
}
