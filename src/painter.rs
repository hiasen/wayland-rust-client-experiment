use crate::buffer;


pub struct Painter {
    float_offset: f32,
    last_frame: u32,
}


impl Painter {
    const COLOR1: u32 = 0xFF666666;
    const COLOR2: u32 = 0xFFEEEEEE;


    pub fn new() -> Painter {
        Painter {
            float_offset: 0.0,
            last_frame: 0,
        }
    }

    pub fn draw(&self, buffer: &mut buffer::Buffer) {
        let width = buffer.width();
        Self::draw_checkerboard_pattern(buffer, width, self.offset());
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
