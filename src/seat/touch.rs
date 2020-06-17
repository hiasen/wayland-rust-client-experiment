use bitflags::bitflags;
use std::collections::HashMap;

use wayland_client::{
    protocol::wl_touch::{WlTouch, Event::*},
    Main,
};

pub fn handle(touch: &Main<WlTouch>) {
    let mut touch_event = TouchEvent::default();
    touch.quick_assign(move |_, event, _| match event {
        Down { id, x, y, time, serial, .. } => {
            let point = touch_event.get_point(id);
            point.event_mask |= EventMask::DOWN;
            point.surface_x = x;
            point.surface_y = y;
            touch_event.time = time;
            touch_event.serial = serial;
        }
        Up { id, .. } => {
            let point = touch_event.get_point(id);
            point.event_mask |= EventMask::UP;
        }
        Motion { id, x, y, time, .. } => {
            let point = touch_event.get_point(id);
            point.event_mask |= EventMask::MOTION;
            point.surface_x = x;
            point.surface_y = y;
            touch_event.time = time;
        }
        Cancel { .. } => {
            touch_event.event_mask |= EventMask::CANCEL;
        }
        Shape { id, major, minor, .. } => {
            let point = touch_event.get_point(id);
            point.event_mask |= EventMask::SHAPE;
            point.major = major;
            point.minor = minor;
        }
        Orientation { id, orientation, .. } => {
            let point = touch_event.get_point(id);
            point.event_mask |= EventMask::ORIENTATION;
            point.orientation = orientation;
        }
        Frame { .. } => {
            eprintln!("{}", touch_event);
            touch_event = Default::default();
        }
        _ => (),
    });
}

bitflags! {
    #[derive(Default)]
    struct EventMask: u32 {
        const DOWN = 1 << 0;
        const UP = 1 << 1;
        const MOTION = 1 << 2;
        const CANCEL = 1 << 3;
        const SHAPE = 1 << 4;
        const ORIENTATION = 1 << 5;
    }
}

#[derive(Default, Debug)]
struct TouchPoint {
    valid: bool,
    id: i32,
    event_mask: EventMask,
    surface_x: f64,
    surface_y: f64,
    major: f64,
    minor: f64,
    orientation: f64,
}

#[derive(Default, Debug)]
struct TouchEvent {
    event_mask: EventMask,
    time: u32,
    serial: u32,
    points: HashMap<i32, TouchPoint>,
}

impl TouchEvent {
    fn get_point(&mut self, id: i32) -> &mut TouchPoint {
        self.points
            .entry(id)
            .or_default()
    }
}

impl std::fmt::Display for TouchEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "touch event @ {}:", self.time)?;

        for (id, point) in self.points.iter() {
            write!(f, "point {}: ", id)?;

            if self.event_mask.contains(EventMask::DOWN) {
                write!(f, "down {},{} ", point.surface_x, point.surface_y)?;
            }

            if self.event_mask.contains(EventMask::UP) {
                write!(f, "up ")?;
            }

            if self.event_mask.contains(EventMask::MOTION) {
                write!(f, "motion {},{} ", point.surface_x, point.surface_y)?;
            }

            if self.event_mask.contains(EventMask::SHAPE) {
                write!(f, "shape {}x{}", point.major, point.minor)?;
            }

            if self.event_mask.contains(EventMask::ORIENTATION) {
                write!(f, "orientation {} ", point.orientation)?;
            }

            writeln!(f)?;
        }
        Ok(())
    }
}
