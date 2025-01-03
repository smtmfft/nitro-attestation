use anyhow::Error;
use common::command::Command;
use new_host::HostConnection;
use nix::sys::socket::{
    accept, bind, listen, recv, socket, AddressFamily, Backlog, MsgFlags, SockFlag, SockType,
    VsockAddr,
};
use std::any;
use std::os::fd::AsRawFd;
use std::os::unix::io::RawFd;

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const VMADDR_PORT: u32 = 5001;
const LOG_PORT: u32 = 5002;
const BUFFER_SIZE: usize = 1024;

mod new_host;

fn main() -> Result<(), Error> {
    let mut host = HostConnection::listen(VMADDR_CID_ANY, VMADDR_PORT, LOG_PORT).map_err(|e| {
        eprintln!("Error connecting to host: {}", e);
        Error::msg(e.to_string())
    })?;

    // 使用自定义的日志处理函数
    let log_handle = host
        .start_log_receiver(|log| {
            // TODO: use local tracing tool
            println!("[{}] {}: {}", log.timestamp, log.level, log.message);
            // or send to log service
            // send_to_log_service(&log);
        })
        .map_err(|e| {
            eprintln!("Error connecting to host: {}", e);
            Error::msg(e.to_string())
        })?;

    // 发送命令并继续其他操作
    let response = host
        .send_command(Command::ExecuteTask {
            task_id: "task1".to_string(),
            task_type: "task_type".to_string(),
            inputs: Default::default(),
        })
        .map_err(|e| {
            eprintln!("Error connecting to host: {}", e);
            Error::msg(e.to_string())
        })?;

    // 如果需要，可以等待日志接收线程结束
    log_handle.join().unwrap();
    Ok(())
}

fn deprecated_main() -> Result<(), Error> {
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
