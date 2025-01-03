use anyhow::Error;
use nix::sys::socket::VsockAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{process::Command, sync::Mutex};
use vsock::{VsockListener, VsockStream};

const VMADDR_CID_HOST: u32 = 3;
const VSOCK_PORT: u32 = 5000;
const LOG_VSOCK_PORT: u32 = 5001;

pub struct Client {
    tasks: Arc<Mutex<HashMap<String, String>>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self, cmd_port: u32, log_port: u32) -> Result<(), Box<dyn Error>> {
        // 启动命令处理线程
        let tasks = Arc::clone(&self.tasks);
        std::thread::spawn(move || -> Result<(), Box<dyn Error>> {
            let listener = VsockListener::bind(cmd_port)?;

            loop {
                if let Ok((mut stream, _addr)) = listener.accept() {
                    let mut buf = vec![0; 1024];
                    if let Ok(n) = stream.read(&mut buf) {
                        if let Ok(command) = serde_json::from_slice::<Command>(&buf[..n]) {
                            let response = handle_command(&command, &tasks);
                            if let Ok(resp_bytes) = serde_json::to_vec(&response) {
                                let _ = stream.write_all(&resp_bytes);
                            }
                        }
                    }
                }
            }
        });

        // 启动日志处理线程
        let listener = VsockListener::bind(log_port)?;
        if let Ok((stream, _addr)) = listener.accept() {
            self.start_logger(stream);
        }

        Ok(())
    }

    fn start_logger(&self, mut stream: VsockStream) {
        std::thread::spawn(move || loop {
            let log = LogMessage {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                level: "INFO".to_string(),
                message: "Test log message".to_string(),
            };

            if let Ok(log_bytes) = serde_json::to_vec(&log) {
                if stream.write_all(&log_bytes).is_err() {
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        });
    }
}

fn handle_command(command: &Command, tasks: &Arc<Mutex<HashMap<String, String>>>) -> Response {
    match command {
        Command::ExecuteTask { task_id, params } => {
            let mut tasks = tasks.lock().unwrap();
            tasks.insert(task_id.clone(), "running".to_string());

            // 在新线程中处理任务
            let tasks = Arc::clone(&tasks);
            let task_id = task_id.clone();
            std::thread::spawn(move || {
                // 模拟任务处理
                std::thread::sleep(std::time::Duration::from_secs(5));
                let mut tasks = tasks.lock().unwrap();
                tasks.insert(task_id, "completed".to_string());
            });

            Response::TaskStarted {
                task_id: task_id.clone(),
            }
        }
        Command::GetStatus { task_id } => {
            let tasks = tasks.lock().unwrap();
            let status = tasks
                .get(task_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            Response::Status {
                task_id: task_id.clone(),
                status,
            }
        }
    }
}
