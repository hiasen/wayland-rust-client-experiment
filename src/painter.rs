use std::error::Error;
use wayland_client::{
    protocol::{wl_buffer, wl_shm},
    Main,
};

use crate::buffer;


pub struct Painter {
    shm: Main<wl_shm::WlShm>,
    float_offset: f32,
    last_frame: u32,
}


impl Painter {
    const WIDTH: usize = 600;
    const HEIGHT: usize = 400;
    const COLOR1: u32 = 0xFF666666;
    const COLOR2: u32 = 0xFFEEEEEE;


    pub fn new(shm: &Main<wl_shm::WlShm>) -> Painter {
        Painter {
            shm: shm.clone(),
            float_offset: 0.0,
            last_frame: 0,
        }
    }

    pub fn draw(&self) -> Result<wl_buffer::WlBuffer, Box<dyn Error>> {
        let mut buffer = buffer::Buffer::new(&self.shm, Self::WIDTH, Self::HEIGHT)?;
        Self::draw_checkerboard_pattern(&mut buffer, Self::WIDTH, self.offset());
        Ok(buffer.wl_buffer().clone())
    }

    fn draw_checkerboard_pattern(buffer: &mut [u32], width: usize, offset: usize) {
        for (y, row) in buffer.chunks_exact_mut(width).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = if ((x + offset) + (y + offset) / 8 * 8) % 16 < 8 {
                    Self::COLOR1
                } else {
                    Self::COLOR2
                };
            }
        }
    }

    fn offset(&self) -> usize {
        (self.float_offset as usize) % 8
    }

    pub fn update_time(&mut self, time: u32) {
        if self.last_frame != 0 {
            let elapsed = time - self.last_frame;
            self.float_offset += (elapsed as f32) / 1000.0 * 24.0;
        }
        self.last_frame = time;
    }
}
