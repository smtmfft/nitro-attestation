use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use vsock::{VsockAddr, VsockListener, VsockStream};

use common::command::{Command, LogMessage, Response};

// Host端代码
pub struct HostConnection {
    cmd_stream: VsockStream,
    log_stream: VsockStream,
}

impl HostConnection {
    pub fn connect(enclave_cid: u32, cmd_port: u32, log_port: u32) -> Result<Self, Box<dyn Error>> {
        let cmd_stream = VsockStream::connect(&VsockAddr::new(enclave_cid, cmd_port))?;
        let log_stream = VsockStream::connect(&VsockAddr::new(enclave_cid, log_port))?;

        Ok(Self {
            cmd_stream,
            log_stream,
        })
    }

    pub fn listen(cid: u32, cmd_port: u32, log_port: u32) -> Result<Self, Box<dyn Error>> {
        // 创建命令通道监听器
        let cmd_listener = VsockListener::bind(&VsockAddr::new(cid, cmd_port))?;
        println!("Listening for command connection on port {}", cmd_port);

        // 等待命令连接
        let (cmd_stream, _addr) = cmd_listener.accept()?;
        println!("Command connection established");

        // 创建日志通道监听器
        let log_listener = VsockListener::bind(&VsockAddr::new(cid, log_port))?;
        println!("Listening for log connection on port {}", log_port);

        // 等待日志连接
        let (log_stream, _addr) = log_listener.accept()?;
        println!("Log connection established");

        Ok(Self {
            cmd_stream,
            log_stream,
        })
    }

    pub fn send_command(&mut self, command: Command) -> Result<Response, Box<dyn Error>> {
        let cmd_bytes = serde_json::to_vec(&command)?;
        self.cmd_stream.write_all(&cmd_bytes)?;

        let mut buf = vec![0; 1024];
        let n = self.cmd_stream.read(&mut buf)?;
        let response: Response = serde_json::from_slice(&buf[..n])?;

        Ok(response)
    }

    pub fn start_log_receiver<F>(
        &mut self,
        mut callback: F,
    ) -> Result<std::thread::JoinHandle<()>, Box<dyn Error>>
    where
        F: FnMut(LogMessage) + Send + 'static,
    {
        let mut log_stream = self.log_stream.try_clone()?;

        let handle = std::thread::spawn(move || {
            let mut buf = vec![0; 1024];
            loop {
                match log_stream.read(&mut buf) {
                    Ok(0) => break, // 连接关闭
                    Ok(n) => {
                        if let Ok(log) = serde_json::from_slice::<LogMessage>(&buf[..n]) {
                            callback(log);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(handle)
    }
}
