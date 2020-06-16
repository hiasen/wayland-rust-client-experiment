use wayland_client::{
    protocol::wl_registry::WlRegistry, AnonymousObject, Attached, DispatchData, GlobalEvent, Main,
    RawEvent,
};

pub fn print_global_event(
    event: GlobalEvent,
    _registry: Attached<WlRegistry>,
    _data: DispatchData,
) {
    match event {
        GlobalEvent::New {
            id,
            interface,
            version,
        } => {
            eprintln!("New global: {} id={} (v{})", interface, id, version);
        }
        GlobalEvent::Removed { id, interface } => {
            eprintln!("Removed global: {} id={}", interface, id);
        }
    }
}

#[allow(dead_code)]
pub fn print_unfiltered_events(event: RawEvent, _obj: Main<AnonymousObject>, _data: DispatchData) {
    eprintln!("Uncaught event: {}::{}", event.interface, event.name);
}
