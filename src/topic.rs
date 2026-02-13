use std::collections::HashMap;

/// 主题订阅结构体
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicSubscription {
    /// 客户端ID
    pub client_id: String,
    /// 订阅的主题
    pub topic: String,
    /// QoS等级
    pub qos: u8,
}

/// 主题节点结构体
#[derive(Debug, Clone)]
pub struct TopicNode {
    /// 主题名称
    pub topic: String,
    /// 子节点
    pub children: HashMap<String, TopicNode>,
    /// 订阅者列表
    pub subscribers: Vec<TopicSubscription>,
}

impl TopicNode {
    /// 创建新的主题节点
    pub fn new(topic: String) -> Self {
        Self {
            topic,
            children: HashMap::new(),
            subscribers: Vec::new(),
        }
    }
}

/// 主题管理器结构体
#[derive(Debug, Clone)]
pub struct TopicManager {
    /// 根节点
    pub root: TopicNode,
}

impl TopicManager {
    /// 创建新的主题管理器
    pub fn new() -> Self {
        Self {
            root: TopicNode::new("root".to_string()),
        }
    }

    /// 添加订阅
    pub async fn add_subscription(&mut self, client_id: String, topic: String, qos: u8) {
        let mut current = &mut self.root;
        let parts: Vec<&str> = topic.split('/').collect();

        for part in parts {
            current = current.children.entry(part.to_string()).or_insert_with(|| TopicNode::new(part.to_string()));
        }

        // 检查是否已经存在相同的订阅
        let existing_subscription = current.subscribers.iter().find(|s| s.client_id == client_id);
        if existing_subscription.is_none() {
            current.subscribers.push(TopicSubscription {
                client_id,
                topic: topic.clone(),
                qos,
            });
        }
    }

    /// 移除订阅
    pub async fn remove_subscription(&mut self, client_id: String, topic: String) {
        let mut current = &mut self.root;
        let parts: Vec<&str> = topic.split('/').collect();

        for part in parts {
            if let Some(child) = current.children.get_mut(part) {
                current = child;
            } else {
                return;
            }
        }

        // 移除订阅
        current.subscribers.retain(|s| s.client_id != client_id);
    }

    /// 查找订阅者
    pub async fn find_subscribers(&self, topic: &str) -> Vec<TopicSubscription> {
        let mut subscribers = Vec::new();
        let parts: Vec<&str> = topic.split('/').collect();

        // 精确匹配
        self.match_topic(&self.root, &parts, 0, &mut subscribers);

        subscribers
    }

    /// 匹配主题
    fn match_topic(&self, node: &TopicNode, parts: &[&str], index: usize, subscribers: &mut Vec<TopicSubscription>) {
        if index == parts.len() {
            // 找到匹配的主题，添加订阅者
            subscribers.extend(node.subscribers.clone());
            return;
        }

        let part = parts[index];

        // 匹配子节点
        if let Some(child) = node.children.get(part) {
            self.match_topic(child, parts, index + 1, subscribers);
        }

        // 匹配通配符 #
        if let Some(child) = node.children.get("#") {
            subscribers.extend(child.subscribers.clone());
        }

        // 匹配通配符 +
        if let Some(child) = node.children.get("+") {
            self.match_topic(child, parts, index + 1, subscribers);
        }
    }
}
