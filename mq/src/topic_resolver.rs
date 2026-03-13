use anyhow::Result;
use regex::Regex;

/// Topic 解析器服务
pub struct TopicResolver {
    /// Topic 格式模板
    topic_pattern: String,
    /// 用于解析 topic 的正则表达式
    topic_regex: Regex,
    /// 节点 ID 分组索引
    node_id_group: usize,
    /// 设备 ID 分组索引
    device_id_group: usize,
    /// 分区 ID 分组索引
    partition_group: Option<usize>,
}

impl TopicResolver {
    /// 创建新的 Topic 解析器
    /// 
    /// # 参数
    /// - `topic_pattern`: Topic 格式模板，使用 `{node_id}`、`{device_id}` 和可选的 `{partition}` 作为占位符
    /// 
    /// # 示例
    /// ```
    /// // 创建解析器，topic 格式为 "{node_id}/{device_id}/data"
    /// ```
    pub fn new(topic_pattern: &str) -> Result<Self> {
        // 构建正则表达式
        let regex_pattern = topic_pattern
            .replace("{node_id}", "([^/]+)")
            .replace("{device_id}", "([^/]+)")
            .replace("{partition}", "([0-9]+)");
        
        let topic_regex = Regex::new(&regex_pattern)?;
        
        // 确定分组索引
        let mut node_id_group = 0;
        let mut device_id_group = 0;
        let mut partition_group = None;
        
        let mut group_index = 1;
        for part in topic_pattern.split('/') {
            if part == "{node_id}" {
                node_id_group = group_index;
                group_index += 1;
            } else if part == "{device_id}" {
                device_id_group = group_index;
                group_index += 1;
            } else if part == "{partition}" {
                partition_group = Some(group_index);
                group_index += 1;
            }
        }
        
        Ok(Self {
            topic_pattern: topic_pattern.to_string(),
            topic_regex,
            node_id_group,
            device_id_group,
            partition_group,
        })
    }
    
    /// 根据 node_id 和 device_id 生成 topic
    /// 
    /// # 参数
    /// - `node_id`: 节点 ID
    /// - `device_id`: 设备 ID
    /// - `partition`: 可选的分区 ID
    /// 
    /// # 返回
    /// 生成的 topic 字符串
    pub fn generate_topic(&self, node_id: &str, device_id: &str, partition: Option<usize>) -> String {
        let mut topic = self.topic_pattern.clone();
        
        // 替换占位符
        topic = topic.replace("{node_id}", node_id);
        topic = topic.replace("{device_id}", device_id);
        
        // 替换分区占位符
        if let Some(p) = partition {
            topic = topic.replace("{partition}", &p.to_string());
        } else {
            // 如果没有提供分区，但模板中有分区占位符，则移除该部分
            topic = topic.replace("/{partition}", "");
        }
        
        topic
    }
    
    /// 从 topic 中解析出 node_id 和 device_id
    /// 
    /// # 参数
    /// - `topic`: 要解析的 topic 字符串
    /// 
    /// # 返回
    /// 包含 node_id、device_id 和可选分区 ID 的元组
    pub fn parse_topic(&self, topic: &str) -> Result<(String, String, Option<usize>)> {
        if let Some(captures) = self.topic_regex.captures(topic) {
            let node_id = captures.get(self.node_id_group)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse node_id from topic"))?
                .as_str()
                .to_string();
            
            let device_id = captures.get(self.device_id_group)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse device_id from topic"))?
                .as_str()
                .to_string();
            
            let partition = if let Some(partition_group) = self.partition_group {
                captures.get(partition_group)
                    .and_then(|m| m.as_str().parse().ok())
            } else {
                None
            };
            
            Ok((node_id, device_id, partition))
        } else {
            Err(anyhow::anyhow!("Topic does not match pattern: {}", self.topic_pattern))
        }
    }
    
    /// 获取默认的 Topic 解析器
    /// 
    /// 默认 topic 格式为 "{node_id}/{device_id}"
    pub fn default() -> Self {
        Self::new("{node_id}/{device_id}").unwrap()
    }
}

/// 标准 Topic 格式常量
pub mod standard_topics {
    /// 遥测数据 topic 格式
    pub const TELEMETRY_TOPIC: &str = "{node_id}/{device_id}/telemetry";
    /// 命令 topic 格式
    pub const COMMAND_TOPIC: &str = "{node_id}/{device_id}/command";
    /// 事件 topic 格式
    pub const EVENT_TOPIC: &str = "{node_id}/{device_id}/event";
    /// 响应 topic 格式
    pub const RESPONSE_TOPIC: &str = "{node_id}/{device_id}/response";

}
