use anyhow::Error;
use common::command::{Command, LogMessage, Response, TaskStatus};
use log::debug;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Read};
use vsock::{VsockAddr, VsockListener, VsockStream};

#[cfg(feature = "aws-nitro")]
const VMADDR_CID_HOST: u32 = 3;
#[cfg(not(feature = "aws-nitro"))]
const VMADDR_CID_HOST: u32 = 2;

pub struct Client {
    cmd_stream: VsockStream,
    log_stream: VsockStream,
    tasks: Arc<Mutex<HashMap<String, String>>>,
}

impl Client {
    pub fn new(cmd_port: u32, log_port: u32) -> Result<Self, Error> {
        Ok(Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            cmd_stream: VsockStream::connect(&VsockAddr::new(VMADDR_CID_HOST, cmd_port))?, // 3 是 AWS Nitro Enclave 中的 parent instance
            log_stream: VsockStream::connect(&VsockAddr::new(VMADDR_CID_HOST, log_port))?,
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        // 命令处理循环
        let tasks = Arc::clone(&self.tasks);
        let mut cmd_stream = self.cmd_stream.try_clone().map_err(|e| {
            eprintln!("Error cloning command stream: {}", e);
            Error::msg(e.to_string())
        })?;

        std::thread::spawn(move || {
            let mut buf = vec![0; 1024];
            loop {
                match cmd_stream.read(&mut buf) {
                    Ok(0) => break, // 连接关闭
                    Ok(n) => {
                        if let Ok(command) = serde_json::from_slice::<Command>(&buf[..n]) {
                            let response = handle_command(&command, &tasks);
                            if let Ok(resp_bytes) = serde_json::to_vec(&response) {
                                let _ = cmd_stream.write_all(&resp_bytes);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading command: {}", e);
                        break;
                    }
                }
            }
        });

        // 启动日志处理线程
        let mut log_stream = self.log_stream.try_clone().map_err(|e| {
            eprintln!("Error cloning log stream: {}", e);
            Error::msg(e.to_string())
        })?;
        std::thread::spawn(move || loop {
            let log = LogMessage {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                level: "INFO".to_string(),
                message: "Test log message".to_string(),
            };
            println!("Sending log message: {:?}", log);

            if let Ok(log_bytes) = serde_json::to_vec(&log) {
                if log_stream.write_all(&log_bytes).is_err() {
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        });

        Ok(())
    }
}

fn handle_command(command: &Command, tasks: &Arc<Mutex<HashMap<String, String>>>) -> Response {
    match command {
        Command::ExecuteTask {
            task_id,
            task_type: _,
            inputs: _,
        } => {
            // 获取锁并修改状态
            let mut tasks_guard = tasks.lock().unwrap();
            tasks_guard.insert(task_id.clone(), "running".to_string());

            // 启动任务处理线程
            let tasks = Arc::clone(&tasks);
            let task_id_clone = task_id.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let mut tasks = tasks.lock().unwrap();
                tasks.insert(task_id_clone.to_owned(), "completed".to_string());
            });

            Response::TaskStarted {
                task_id: task_id.clone(),
                estimated_duration: Some(5),
            }
        }
        Command::QueryTask { task_id } => {
            let tasks = tasks.lock().unwrap();
            let status = tasks
                .get(task_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            Response::TaskStatus {
                task_id: task_id.clone(),
                status: match status.as_str() {
                    "running" => TaskStatus::Running,
                    "completed" => TaskStatus::Completed,
                    _ => TaskStatus::Failed,
                },
                progress: None,
                error: None,
                result: None,
            }
        }
        _ => Response::Error {
            code: 400,
            message: "Unsupported command".to_string(),
        },
    }
}
