use anyhow::Result;
use nix::sys::socket::{self, AddressFamily, Backlog, SockFlag, SockType, VsockAddr};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::io::FromRawFd;

const VSOCK_PORT: u32 = 1234;
const VMADDR_CID_ANY: u32 = 4294967295;

struct VsockListener {
    fd: std::os::unix::io::OwnedFd,
}

impl VsockListener {
    fn new() -> Result<Self> {
        let fd = socket::socket(
            AddressFamily::Vsock,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )?;

        let addr = VsockAddr::new(VMADDR_CID_ANY, VSOCK_PORT);
        socket::bind(fd.as_raw_fd() as i32, &addr)?;
        socket::listen(&fd, Backlog::new(1).unwrap())?;

        Ok(Self { fd })
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        let mut stream = unsafe { std::fs::File::from_raw_fd(self.fd.as_raw_fd()) };

        // 首先发送数据长度
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes())?;

        // 然后发送数据
        stream.write_all(data)?;
        stream.flush()?;

        Ok(())
    }
}
