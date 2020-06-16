use bitflags::bitflags;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wayland_client::{
    protocol::{
        wl_pointer::{ButtonState, Event, Event::*, WlPointer},
        wl_seat::WlSeat,
    },
    Main,
};

pub struct Handler {
    current_pointer: Option<WlPointer>,
    seat: Main<WlSeat>,
    pointer_event: Rc<RefCell<PointerEvent>>,
}

impl Handler {
    pub fn new(seat: &Main<WlSeat>) -> Handler {
        Handler {
            current_pointer: None,
            seat: seat.clone(),
            pointer_event: Default::default(),
        }
    }

    pub fn status_update(&mut self, have_pointer: bool) {
        if have_pointer && self.current_pointer.is_none() {
            let pointer = self.seat.get_pointer();
            let pointer_event = self.pointer_event.clone();
            pointer.quick_assign(move |_, event, _| {
                let mut pointer_event = pointer_event.borrow_mut();
                if let Frame = event {
                    eprintln!("{}", pointer_event);
                    *pointer_event = Default::default();
                } else {
                    pointer_event.update(event);
                }
            });
            self.current_pointer = Some(pointer.detach());
        } else if !have_pointer && self.current_pointer.is_some() {
            self.current_pointer.take().unwrap().release();
        }
    }
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
    fn update(&mut self, event: Event) {
        match event {
            Enter {
                serial,
                surface_x,
                surface_y,
                ..
            } => {
                self.event_mask |= EventMask::ENTER;
                self.serial = serial;
                self.surface_x = surface_x;
                self.surface_y = surface_y;
            }
            Leave { serial, .. } => {
                self.event_mask |= EventMask::LEAVE;
                self.serial = serial;
            }
            Motion {
                time,
                surface_x,
                surface_y,
            } => {
                self.event_mask |= EventMask::MOTION;
                self.time = time;
                self.surface_x = surface_x;
                self.surface_y = surface_y;
            }
            Button {
                serial,
                time,
                button,
                state,
            } => {
                self.event_mask |= EventMask::BUTTON;
                self.time = time;
                self.serial = serial;
                self.button = button;
                self.state = state.to_raw();
            }
            Axis { time, axis, value } => {
                self.event_mask |= EventMask::AXIS;
                self.time = time;
                self.axes[axis.to_raw() as usize].valid = true;
                self.axes[axis.to_raw() as usize].value = value;
            }
            AxisSource { axis_source } => {
                self.event_mask |= EventMask::AXIS_SOURCE;
                self.axis_source = axis_source.to_raw();
            }
            AxisStop { time, axis } => {
                self.event_mask |= EventMask::AXIS_STOP;
                self.time = time;
                self.axes[axis.to_raw() as usize].valid = true;
            }
            AxisDiscrete { axis, discrete } => {
                self.event_mask |= EventMask::AXIS_DISCRETE;
                self.axes[axis.to_raw() as usize].valid = true;
                self.axes[axis.to_raw() as usize].discrete = discrete;
            }
            _ => (),
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

        let axis_events = EventMask::AXIS
            | EventMask::AXIS_SOURCE
            | EventMask::AXIS_STOP
            | EventMask::AXIS_DISCRETE;

        let axis_name = |i| {
            use wayland_client::protocol::wl_pointer::Axis;
            if let Option::Some(Axis::VerticalScroll) = Axis::from_raw(i) {
                "vertical"
            } else {
                "horizontal"
            }
        };
        let axis_source = |x| {
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
        };

        if self.event_mask.intersects(axis_events) {
            for (i, a) in self.axes.iter().enumerate() {
                if !a.valid {
                    continue;
                }
                write!(f, "{} axis ", axis_name(i as u32))?;
                if self.event_mask.contains(EventMask::AXIS) {
                    write!(f, "value {} ", a.value)?;
                }
                if self.event_mask.contains(EventMask::AXIS_DISCRETE) {
                    write!(f, "discrete {} ", a.discrete)?;
                }
                if self.event_mask.contains(EventMask::AXIS_SOURCE) {
                    write!(f, "via {} ", axis_source(self.axis_source))?;
                }
                if self.event_mask.contains(EventMask::AXIS_STOP) {
                    write!(f, "(stopped) ")?;
                }
            }
        }
        Ok(())
    }
}
