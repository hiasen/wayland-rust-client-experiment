use wayland_client::{
    protocol::{wl_buffer, wl_shm},
    Main,
};
use std::error::Error;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::os::unix::io::AsRawFd;
use crate::shared_memory;

pub struct Buffer {
    width: usize,
    height: usize,
    wl_buffer: wl_buffer::WlBuffer,
    mmap: shared_memory::MemMap,
}

impl Buffer {
    pub fn new(shm: &Main<wl_shm::WlShm>, width: usize, height: usize) -> Result<Self, Box<dyn Error>> {
        let stride = size_of::<u32>() * width;
        let size = height * stride;
        let mmap = shared_memory::MemMap::anon_file(size)?;

        assert!(Self::is_aligned(&mmap), "{:?} is not sufficiently aligned!", mmap);

        let pool = shm
            .create_pool(mmap.backing_file().as_raw_fd(), size as i32);
        let wl_buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Xrgb8888,
        );
        pool.destroy();
        wl_buffer.quick_assign(|buffer, event, _| match event {
            wl_buffer::Event::Release => buffer.destroy(),
            _ => (),
        });

        Ok(Self {
            width,
            height,
            wl_buffer: wl_buffer.detach(),
            mmap,
        })
    }

    fn is_aligned(buf: &[u8]) -> bool {
        let (prefix, _, postfix) = unsafe { buf.align_to::<u32>() };
        prefix.len() == 0 && postfix.len() == 0
    }

    pub fn wl_buffer(&self) -> &wl_buffer::WlBuffer {
        &self.wl_buffer
    }

    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }
}


impl Deref for Buffer {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { (&self.mmap).align_to() }.1
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (&mut self.mmap).align_to_mut() }.1
    }
}
