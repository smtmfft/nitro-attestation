use aws_nitro_enclaves_vsock::{VsockStream, VsockListener};
use std::error::Error;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// 共享的类型定义
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    ExecuteTask { task_id: String, params: Vec<String> },
    GetStatus { task_id: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    TaskStarted { task_id: String },
    TaskCompleted { task_id: String, result: String },
    TaskFailed { task_id: String, error: String },
    Status { task_id: String, status: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
}

// Host端代码
pub struct HostConnection {
    cmd_stream: VsockStream,
    log_stream: VsockStream,
}

impl HostConnection {
    pub fn connect(enclave_cid: u32, cmd_port: u32, log_port: u32) -> Result<Self, Box<dyn Error>> {
        let cmd_stream = VsockStream::connect(enclave_cid, cmd_port)?;
        let log_stream = VsockStream::connect(enclave_cid, log_port)?;
        
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

    pub fn start_log_receiver<F>(&mut self, mut callback: F) -> Result<std::thread::JoinHandle<()>, Box<dyn Error>> 
    where 
        F: FnMut(LogMessage) + Send + 'static 
    {
        let mut log_stream = self.log_stream.try_clone()?;
        
        let handle = std::thread::spawn(move || {
            let mut buf = vec![0; 1024];
            loop {
                match log_stream.read(&mut buf) {
                    Ok(0) => break,  // 连接关闭
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
