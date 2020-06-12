use std::fs::File;
use std::ffi::CString;
use nix::sys::memfd;
use std::os::unix::io::FromRawFd;

pub fn create_anonymous_file() -> Result<File,  Box<dyn std::error::Error>> {
    let fd = memfd::memfd_create(
        CString::new("Fafa")?.as_c_str(),
        memfd::MemFdCreateFlag::empty()
    )?;
    Ok(unsafe {File::from_raw_fd(fd)})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let _file = create_anonymous_file();
    }
}