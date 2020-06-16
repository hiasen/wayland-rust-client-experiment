use memmap2;
use nix::sys::memfd;
use std::ffi::CString;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::os::unix::io::FromRawFd;

pub fn create_anonymous_file() -> Result<File, Box<dyn std::error::Error>> {
    let fd = memfd::memfd_create(
        CString::new("Fafa")?.as_c_str(),
        memfd::MemFdCreateFlag::empty(),
    )?;
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub struct MemMap {
    buffer: memmap2::MmapMut,
    file: File,
}

impl MemMap {
    pub fn anon_file(size: usize) -> Result<MemMap, Box<dyn std::error::Error>> {
        let file = create_anonymous_file()?;
        file.set_len(size as u64)?;
        let buffer = unsafe {
            memmap2::MmapOptions::new()
                .len(size as usize)
                .map_mut(&file)?
        };
        Ok(MemMap { buffer, file })
    }
    pub fn backing_file(&self) -> File {
        self.file.try_clone().unwrap()
    }
}

impl Deref for MemMap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl DerefMut for MemMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_anonymous_file() {
        let _file = create_anonymous_file();
    }
    #[test]
    fn create_anon() {
        MemMap::anon_file(10).unwrap();
    }
    #[test]
    fn memmap_deref() {
        let m = MemMap::anon_file(10).unwrap();
        let _b: &[u8] = &m;
    }

    #[test]
    fn derefed_buffer_has_correct_size() {
        let size = 10;
        let m = MemMap::anon_file(size).unwrap();
        let b = &m;
        assert_eq!(size, b.len());
    }

    #[test]
    fn backing_file() {
        let m = MemMap::anon_file(10).unwrap();
        let _file: File = m.backing_file();
    }
}
