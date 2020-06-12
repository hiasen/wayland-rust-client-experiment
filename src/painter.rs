use super::shared_memory;
use std::error::Error;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use wayland_client::{
    protocol::{wl_buffer, wl_shm},
    Main,
};

pub struct Painter {
    shm: Main<wl_shm::WlShm>,
    offset: f32,
    last_frame: u32,
}
impl Painter {
    pub fn new(shm: Main<wl_shm::WlShm>) -> Painter {
        Painter {
            shm,
            offset: 0.0,
            last_frame: 0,
        }
    }
    pub fn draw(&self) -> Result<wl_buffer::WlBuffer, Box<dyn Error>> {
        let width = 600;
        let height = 400;
        let stride = 4 * width;
        let size = height * stride;
        let mut file = shared_memory::create_anonymous_file()?;
        let offset = (self.offset as i32) % 8;
        let color1 = (0xFF666666 as u32).to_le_bytes();
        let color2 = (0xFFEEEEEE as u32).to_le_bytes();
        for y in 0..height {
            for x in 0..width {
                file.write(if ((x + offset) + (y + offset) / 8 * 8) % 16 < 8 {
                    &color1
                } else {
                    &color2
                })?;
            }
        }
        let pool = self.shm.create_pool(file.as_raw_fd(), size);
        let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Xrgb8888);
        pool.destroy();
        buffer.quick_assign(|buffer, event, _| {
            if let wl_buffer::Event::Release = event {
                buffer.destroy();
            }
        });
        Ok(buffer.detach())
    }
    pub fn update_time(&mut self, time: u32) {
        if self.last_frame != 0 {
            let elapsed = time - self.last_frame;
            self.offset += (elapsed as f32) / 1000.0 * 24.0;
        }
        self.last_frame = time;
    }
}
