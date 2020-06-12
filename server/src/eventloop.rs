use calloop::generic::Generic;
use calloop::signals::{Signal::SIGINT, Signals};
use calloop::{EventLoop, Interest, Mode};
use wayland_server::Display;

pub fn run(mut display: Display) {
    let mut event_loop: EventLoop<()> =
        EventLoop::new().expect("Failed to initialize the event loop!");
    let handle = event_loop.handle();
    let source = Generic::from_fd(display.get_poll_fd(), Interest::Both, Mode::Level);

    handle
        .insert_source(source, move |_, _, _| {
            display.dispatch(std::time::Duration::from_secs(0), &mut ())
                .expect("Error in display.dispatch.");
            display.flush_clients(&mut ());
            Ok(())
        })
        .expect("Failed to insert source");

    let quit_signal = event_loop.get_signal();

    let source = Signals::new(&[SIGINT]).expect("Failed to create source");
    handle
        .insert_source(source, move |_, _, _| {
            quit_signal.stop();
            ()
        })
        .expect("Failed to insert source");

    event_loop
        .run(None, &mut (), move |_| {})
        .expect("Error in event_loop.run");
}
