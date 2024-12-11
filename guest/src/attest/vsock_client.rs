use std::os::fd::AsRawFd;
// client.rs
use nix::sys::socket::{connect, send, socket, AddressFamily, SockFlag, SockType};
use nix::sys::socket::{MsgFlags, VsockAddr};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::vsock_protocol::{MessageType, ProtocolHeader};

const HOST_CID: u32 = 3;
const SERVICE_PORT: u32 = 1234;

pub struct VsockClient {
    sender: mpsc::Sender<Vec<u8>>,
}

impl VsockClient {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            let mut retry_count = 0;
            loop {
                match setup_vsock_connection() {
                    Ok(sock_fd) => {
                        retry_count = 0;
                        while let Ok(data) = receiver.recv() {
                            if let Err(e) = send(sock_fd, &data, MsgFlags::empty()) {
                                eprintln!("Failed to send data: {}", e);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        retry_count += 1;
                        thread::sleep(Duration::from_secs(retry_count.min(60)));
                    }
                }
            }
        });

        VsockClient { sender }
    }

    pub fn log(&self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] {}\n", timestamp, message);

        let header = ProtocolHeader::new(MessageType::Log, log_entry.len() as u32);

        let mut data = header.to_vec();
        data.extend_from_slice(log_entry.as_bytes());

        let _ = self.sender.send(data);
    }

    pub fn send_file(&self, path: PathBuf) -> std::io::Result<()> {
        let mut file = File::open(&path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 先发送文件名
        let name_header = ProtocolHeader::new(MessageType::FileTransfer, filename.len() as u32);
        let mut name_data = name_header.to_vec();
        name_data.extend_from_slice(filename.as_bytes());
        let _ = self.sender.send(name_data);

        // 读取并发送文件内容
        let mut buffer = [0; 8192];
        loop {
            match file.read(&mut buffer)? {
                0 => break,
                n => {
                    let content_header = ProtocolHeader::new(MessageType::FileTransfer, n as u32);
                    let mut content_data = content_header.to_vec();
                    content_data.extend_from_slice(&buffer[..n]);
                    let _ = self.sender.send(content_data);
                }
            }
        }
        Ok(())
    }
}

fn setup_vsock_connection() -> Result<i32, Box<dyn std::error::Error>> {
    let sock_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )?;

    let addr = VsockAddr::new(HOST_CID, SERVICE_PORT);
    connect(sock_fd.as_raw_fd(), &addr)?;

    Ok(sock_fd.as_raw_fd())
}

// 方便使用的宏
#[macro_export]
macro_rules! vsock_log {
    ($client:expr, $($arg:tt)*) => {
        $client.log(&format!($($arg)*));
    }
}
