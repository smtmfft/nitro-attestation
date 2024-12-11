use std::io::{self};
use nix::sys::socket::{self, MsgFlags};
use std::os::unix::io::RawFd;

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Log = 1,
    FileTransfer = 2,
}

#[derive(Debug)]
pub struct ProtocolHeader {
    msg_type: u8,
    payload_size: u32,
}

impl ProtocolHeader {
    pub fn new(msg_type: MessageType, payload_size: u32) -> Self {
        Self {
            msg_type: msg_type as u8,
            payload_size,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(5);
        buffer.push(self.msg_type);
        buffer.extend_from_slice(&self.payload_size.to_be_bytes());
        buffer
    }

    pub fn read_from(fd: RawFd) -> io::Result<Option<Self>> {
        let mut header_buf = [0u8; 5];
        match socket::recv(fd, &mut header_buf, MsgFlags::empty()) {
            Ok(5) => {
                let msg_type = header_buf[0];
                let mut size_bytes = [0u8; 4];
                size_bytes.copy_from_slice(&header_buf[1..5]);
                let payload_size = u32::from_be_bytes(size_bytes);
                
                Ok(Some(Self {
                    msg_type,
                    payload_size,
                }))
            }
            Ok(0) => Ok(None),  // Connection closed
            Ok(_) => Err(io::Error::new(io::ErrorKind::Other, "Incomplete header")),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }
}

pub fn read_exact(fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
    let mut total_read = 0;
    while total_read < buf.len() {
        match socket::recv(fd, &mut buf[total_read..], MsgFlags::empty()) {
            Ok(0) => break,  // Connection closed
            Ok(n) => total_read += n,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }
    Ok(total_read)
}

pub fn send_message(fd: RawFd, msg_type: MessageType, payload: &[u8]) -> io::Result<()> {
    let header = ProtocolHeader::new(msg_type, payload.len() as u32);
    let header_bytes = header.to_vec();
    
    // Send header
    socket::send(fd, &header_bytes, MsgFlags::empty())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    // Send payload
    socket::send(fd, payload, MsgFlags::empty())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    Ok(())
}

pub fn read_message(fd: RawFd) -> io::Result<Option<(MessageType, Vec<u8>)>> {
    match ProtocolHeader::read_from(fd)? {
        Some(header) => {
            let mut payload = vec![0u8; header.payload_size as usize];
            read_exact(fd, &mut payload)?;
            
            let msg_type = match header.msg_type {
                1 => MessageType::Log,
                2 => MessageType::FileTransfer,
                _ => return Err(io::Error::new(io::ErrorKind::Other, "Unknown message type"))
            };
            
            Ok(Some((msg_type, payload)))
        }
        None => Ok(None)
    }
}