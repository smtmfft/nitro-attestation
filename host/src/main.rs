use nix::sys::socket::{
    accept, bind, listen, recv, socket, AddressFamily, Backlog, MsgFlags, SockFlag, SockType,
    VsockAddr,
};
use std::io::Error;
use std::os::fd::AsRawFd;
use std::os::unix::io::RawFd;

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const VMADDR_PORT: u32 = 1234;
const BUFFER_SIZE: usize = 1024;

fn main() -> Result<(), Error> {
    // 创建 VSOCK socket
    let sock_fd = socket(
        AddressFamily::Vsock, // VSOCK 地址族
        SockType::Stream,     // 流式套接字
        SockFlag::empty(),    // 没有额外标志
        None,                 // 默认协议
    )?;

    let sock_i32_fd = sock_fd.as_raw_fd();

    // 准备服务器地址
    let sockaddr = VsockAddr::new(VMADDR_CID_ANY, VMADDR_PORT);

    // 绑定socket到地址
    bind(sock_fd.as_raw_fd(), &sockaddr)?;

    // 开始监听连接，最大队列长度为5
    listen(&sock_fd, Backlog::new(5).unwrap())?;
    println!("Server listening on port {}", VMADDR_PORT);

    loop {
        // 接受新的连接
        match accept(sock_i32_fd) {
            Ok(client_fd) => {
                println!("New client connected");
                handle_client(client_fd)?;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                continue;
            }
        }
    }
}

fn handle_client(client_fd: RawFd) -> Result<(), Error> {
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        // 从客户端接收数据
        match recv(client_fd, &mut buffer, MsgFlags::empty()) {
            Ok(0) => {
                // 客户端关闭连接
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                // 打印接收到的数据
                if let Ok(message) = String::from_utf8(buffer[..n].to_vec()) {
                    println!("Received: {}", message);
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }

    // 使用 close() 关闭客户端连接
    nix::unistd::close(client_fd)?;
    Ok(())
}
