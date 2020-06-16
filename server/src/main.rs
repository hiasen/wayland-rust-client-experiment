use wayland_server::Display;
mod eventloop;

fn main() {
    let mut display = Display::new();
    let socket = display
        .add_socket_auto()
        .expect("Unable to add socket to Wayland display.");

    let socket = socket.to_str().expect("Not valid string");

    eprintln!("Running socket on {}", socket);
    eventloop::run(display);
}
