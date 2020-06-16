use super::shared_memory;
use std::error::Error;
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
    pub fn new(shm: &Main<wl_shm::WlShm>) -> Painter {
        Painter {
            shm: shm.clone(),
            offset: 0.0,
            last_frame: 0,
        }
    }
    pub fn draw(&self) -> Result<wl_buffer::WlBuffer, Box<dyn Error>> {
        let width = 600;
        let height = 400;
        let stride = 4 * width;
        let size = height * stride;
        let mut buffer = shared_memory::MemMap::anon_file(size)?;
        let u32_buffer = u8_to_u32_slice(&mut buffer);
        self.draw_checkerboard_pattern(u32_buffer, width, (self.offset as usize) % 8);

        let pool = self
            .shm
            .create_pool(buffer.backing_file().as_raw_fd(), size as i32);
        let wl_buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Xrgb8888,
        );
        pool.destroy();
        wl_buffer.quick_assign(|buffer, event, _| {
            if let wl_buffer::Event::Release = event {
                buffer.destroy();
            }
        });
        Ok(wl_buffer.detach())
    }

    fn draw_checkerboard_pattern(&self, buffer: &mut [u32], width: usize, offset: usize) {
        for (y, row) in buffer.chunks_exact_mut(width).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = if ((x + offset) + (y + offset) / 8 * 8) % 16 < 8 {
                    0xFF666666
                } else {
                    0xFFEEEEEE
                };
            }
        }
    }
    pub fn update_time(&mut self, time: u32) {
        if self.last_frame != 0 {
            let elapsed = time - self.last_frame;
            self.offset += (elapsed as f32) / 1000.0 * 24.0;
        }
        self.last_frame = time;
    }
}

fn u8_to_u32_slice(x: &mut [u8]) -> &mut [u32] {
    assert!(x.len() % 4 == 0);
    let ptr = x.as_mut_ptr();
    let ptr = unsafe { std::mem::transmute(ptr) };
    unsafe { std::slice::from_raw_parts_mut(ptr, x.len() / 4) }
}
