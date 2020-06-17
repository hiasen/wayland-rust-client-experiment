use bitflags::bitflags;
use std::fmt;
use wayland_client::{
    protocol::wl_pointer::{ButtonState, Event::*, WlPointer},
    Main,
};

pub fn handle(pointer: &Main<WlPointer>) {
    let mut pointer_event = PointerEvent::default();
    pointer.quick_assign(move |_pointer, event, _data| match event {
        Enter {
            serial,
            surface_x,
            surface_y,
            ..
        } => {
            pointer_event.event_mask |= EventMask::ENTER;
            pointer_event.serial = serial;
            pointer_event.surface_x = surface_x;
            pointer_event.surface_y = surface_y;
        }
        Leave { serial, .. } => {
            pointer_event.event_mask |= EventMask::LEAVE;
            pointer_event.serial = serial;
        }
        Motion {
            time,
            surface_x,
            surface_y,
        } => {
            pointer_event.event_mask |= EventMask::MOTION;
            pointer_event.time = time;
            pointer_event.surface_x = surface_x;
            pointer_event.surface_y = surface_y;
        }
        Button {
            serial,
            time,
            button,
            state,
        } => {
            pointer_event.event_mask |= EventMask::BUTTON;
            pointer_event.time = time;
            pointer_event.serial = serial;
            pointer_event.button = button;
            pointer_event.state = state.to_raw();
        }
        Axis { time, axis, value } => {
            pointer_event.event_mask |= EventMask::AXIS;
            pointer_event.time = time;
            pointer_event.axes[axis.to_raw() as usize].valid = true;
            pointer_event.axes[axis.to_raw() as usize].value = value;
        }
        AxisSource { axis_source } => {
            pointer_event.event_mask |= EventMask::AXIS_SOURCE;
            pointer_event.axis_source = axis_source.to_raw();
        }
        AxisStop { time, axis } => {
            pointer_event.event_mask |= EventMask::AXIS_STOP;
            pointer_event.time = time;
            pointer_event.axes[axis.to_raw() as usize].valid = true;
        }
        AxisDiscrete { axis, discrete } => {
            pointer_event.event_mask |= EventMask::AXIS_DISCRETE;
            pointer_event.axes[axis.to_raw() as usize].valid = true;
            pointer_event.axes[axis.to_raw() as usize].discrete = discrete;
        }
        Frame => {
            eprintln!("{}", pointer_event);
            pointer_event = Default::default();
        }
        _ => (),
    });
}

bitflags! {
    #[derive(Default)]
    struct EventMask: u32 {
        const ENTER = 1 << 0;
        const LEAVE = 1 << 1;
        const MOTION = 1 << 2;
        const BUTTON = 1 << 3;
        const AXIS = 1 << 4;
        const AXIS_SOURCE = 1 << 5;
        const AXIS_STOP = 1 << 6;
        const AXIS_DISCRETE = 1 << 7;
        const AXIS_EVENTS = (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7);
    }
}

#[derive(Default, Debug)]
struct Axes {
    valid: bool,
    value: f64,
    discrete: i32,
}

#[derive(Default, Debug)]
struct PointerEvent {
    event_mask: EventMask,
    surface_x: f64,
    surface_y: f64,
    button: u32,
    state: u32,
    time: u32,
    serial: u32,
    axes: [Axes; 2],
    axis_source: u32,
}

impl PointerEvent {
    fn axis_name(i: usize) -> &'static str {
        use wayland_client::protocol::wl_pointer::Axis;
        if let Option::Some(Axis::VerticalScroll) = Axis::from_raw(i as u32) {
            "vertical"
        } else {
            "horizontal"
        }
    }

    fn axis_source_type(x: u32) -> &'static str {
        use wayland_client::protocol::wl_pointer::AxisSource;
        if let Some(source) = AxisSource::from_raw(x) {
            match source {
                AxisSource::Wheel => "wheel",
                AxisSource::Finger => "finger",
                AxisSource::Continuous => "continous",
                AxisSource::WheelTilt => "wheel tilt",
                _ => "unknown source",
            }
        } else {
            "unknown source"
        }
    }
}

impl fmt::Display for PointerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pointer frame @ {}: ", self.time)?;

        if self.event_mask.contains(EventMask::ENTER) {
            write!(f, "entered {}, {} ", self.surface_x, self.surface_y)?;
        }

        if self.event_mask.contains(EventMask::LEAVE) {
            write!(f, "leave")?;
        }

        if self.event_mask.contains(EventMask::MOTION) {
            write!(f, "motion {}, {}", self.surface_x, self.surface_y)?;
        }

        if self.event_mask.contains(EventMask::BUTTON) {
            let state = if self.state == ButtonState::Released.to_raw() {
                "released"
            } else {
                "pressed"
            };
            write!(f, "button {}, {}", self.button, state)?;
        }

        if self.event_mask.intersects(EventMask::AXIS_EVENTS) {
            for (i, a) in self.axes.iter().enumerate() {
                if !a.valid {
                    continue;
                }
                write!(f, "{} axis ", Self::axis_name(i))?;
                if self.event_mask.contains(EventMask::AXIS) {
                    write!(f, "value {} ", a.value)?;
                }
                if self.event_mask.contains(EventMask::AXIS_DISCRETE) {
                    write!(f, "discrete {} ", a.discrete)?;
                }
                if self.event_mask.contains(EventMask::AXIS_SOURCE) {
                    write!(f, "via {} ", Self::axis_source_type(self.axis_source))?;
                }
                if self.event_mask.contains(EventMask::AXIS_STOP) {
                    write!(f, "(stopped) ")?;
                }
            }
        }
        Ok(())
    }
}
