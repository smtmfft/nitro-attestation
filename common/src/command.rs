use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // 健康检查
    Ping,

    // 执行任务
    ExecuteTask {
        task_id: String,
        task_type: String,
        inputs: Vec<u8>,
    },

    // 查询任务状态
    QueryTask {
        task_id: String,
    },

    // 取消任务
    CancelTask {
        task_id: String,
    },

    // 获取系统状态
    GetStatus {
        items: Vec<StatusItem>,
    },

    // 更新配置
    UpdateConfig {
        config: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusItem {
    Memory,
    Cpu,
    Tasks,
    Network,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // 健康检查响应
    Pong {
        timestamp: u64,
    },

    // 任务执行响应
    TaskStarted {
        task_id: String,
        estimated_duration: Option<u64>,
    },

    // 任务状态响应
    TaskStatus {
        task_id: String,
        status: TaskStatus,
        progress: Option<u32>,
        error: Option<String>,
        result: Option<String>,
    },

    // 任务取消响应
    TaskCancelled {
        task_id: String,
        success: bool,
        error: Option<String>,
    },

    // 系统状态响应
    Status {
        memory_usage: Option<ResourceUsage>,
        cpu_usage: Option<ResourceUsage>,
        running_tasks: Option<Vec<String>>,
        network_stats: Option<NetworkStats>,
    },

    // 配置更新响应
    ConfigUpdated {
        success: bool,
        error: Option<String>,
    },

    // 通用错误响应
    Error {
        code: u32,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

// 使用示例
impl Command {
    // 创建一个执行任务的命令
    pub fn execute_task(task_id: String, task_type: String) -> Self {
        Command::ExecuteTask {
            task_id,
            task_type,
            inputs: vec![],
        }
    }

    // 创建一个执行任务的命令，带参数
    pub fn execute_task_with_params(
        task_id: String,
        task_type: String,
        parameters: Vec<u8>,
    ) -> Self {
        Command::ExecuteTask {
            task_id,
            task_type,
            inputs: parameters,
        }
    }

    // 创建一个查询任务状态的命令
    pub fn query_task(task_id: String) -> Self {
        Command::QueryTask { task_id }
    }

    // 创建一个获取所有系统状态的命令
    pub fn get_all_status() -> Self {
        Command::GetStatus {
            items: vec![StatusItem::All],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
}

// 使用示例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_commands() {
        // 创建执行任务命令
        let mut params = HashMap::new();
        params.insert("input".to_string(), "data.txt".to_string());
        params.insert("mode".to_string(), "fast".to_string());

        let cmd = Command::execute_task_with_params(
            "task-123".to_string(),
            "data-processing".to_string(),
            params,
        );

        // 创建查询状态命令
        let status_cmd = Command::GetStatus {
            items: vec![StatusItem::Memory, StatusItem::Cpu],
        };

        // 测试序列化
        let serialized = serde_json::to_string(&cmd).unwrap();
        let deserialized: Command = serde_json::from_str(&serialized).unwrap();
    }
}
