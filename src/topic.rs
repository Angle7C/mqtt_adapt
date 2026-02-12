use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// 主题订阅结构体
#[derive(Debug, Clone)]
pub struct TopicSubscription {
    /// 客户端ID
    pub client_id: String,
    /// QoS级别
    pub qos: u8,
}
impl PartialEq for TopicSubscription {
    fn eq(&self, other: &Self) -> bool {
        self.client_id == other.client_id
    }
}
/// 主题树节点
#[derive(Debug, Clone)]
pub struct TopicNode {
    /// 主题名称
    topic_name: String,
    /// 子节点
    children: HashMap<String, TopicNode>,
    /// 订阅者列表
    subscribers: Vec<TopicSubscription>,
    /// 是否为通配符节点（+ 或 #）
    is_wildcard: bool,
}

impl TopicNode {
    /// 创建新的主题节点
    pub fn new(topic_name: String) -> Self {
        Self {
            topic_name,
            children: HashMap::new(),
            subscribers: Vec::new(),
            is_wildcard: false,
        }
    }

    /// 创建通配符节点
    pub fn new_wildcard(topic_name: String) -> Self {
        Self {
            topic_name,
            children: HashMap::new(),
            subscribers: Vec::new(),
            is_wildcard: true,
        }
    }
}

impl PartialEq for TopicNode {
    fn eq(&self, other: &Self) -> bool {
        self.topic_name == other.topic_name
    }
}

/// 主题管理器
#[derive(Debug)]
pub struct TopicManager {
    /// 主题树的根节点
    root: RwLock<TopicNode>,
}

impl TopicManager {
    /// 创建新的主题管理器
    pub fn new() -> Self {
        Self {
            root: RwLock::new(TopicNode::new("root".to_string())),
        }
    }

    /// 添加订阅
    pub async fn add_subscription(&self, topic_filter: &str, client_id: String, qos: u8) {
        let mut root = self.root.write().await;
        let parts: Vec<&str> = topic_filter.split('.').collect();

        let mut current_node = &mut *root;
        for (i, part) in parts.iter().enumerate() {
            let part_str = *part;
            let child_node = current_node
                .children
                .entry(part_str.to_string())
                .or_insert_with(|| {
                    if part_str == "+" || part_str == "#" {
                        TopicNode::new_wildcard(part_str.to_string())
                    } else {
                        TopicNode::new(part_str.to_string())
                    }
                });

            if i == parts.len() - 1 {
                // 到达最后一个部分，添加订阅者
                if !child_node
                    .subscribers
                    .iter()
                    .any(|sub| sub.client_id == client_id)
                {
                    child_node.subscribers.push(TopicSubscription {
                        client_id: client_id.clone(),
                        qos,
                    });
                }
            }

            current_node = child_node;
        }
    }

    /// 添加主题
    pub async fn add_topic(&self, topic_filter: &str) {
        let mut root = self.root.write().await;
        let parts: Vec<&str> = topic_filter.split('.').collect();

        let mut current_node = &mut *root;
        for part in parts {
            let child_node = current_node
                .children
                .entry(part.to_string())
                .or_insert_with(|| {
                    if part == "+" || part == "#" {
                        TopicNode::new_wildcard(part.to_string())
                    } else {
                        TopicNode::new(part.to_string())
                    }
                });
            current_node = child_node;
        }
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, topic_filter: &str, client_id: &str) {
        let mut root = self.root.write().await;
        let parts: Vec<&str> = topic_filter.split('.').collect();

        // 第一步：移除订阅者
        {
            let mut current_node = &mut *root;
            let mut exists = true;

            for part in &parts {
                let part_str = *part;
                if let Some(child_node) = current_node.children.get_mut(part_str) {
                    current_node = child_node;
                } else {
                    exists = false;
                    break;
                }
            }

            if !exists {
                return;
            }

            // 移除订阅者
            current_node
                .subscribers
                .retain(|sub| sub.client_id != client_id);
        }
        Self::clear_empty_nodes(&mut *root);
        // 第二步：删除空节点
    }
    fn clear_empty_nodes(root: &mut TopicNode) {
        if root.subscribers.is_empty() && root.children.is_empty() {
            // 删除空节点
            root.children.clear();
        };
        for child in root.children.values_mut() {
            Self::clear_empty_nodes(child);
        }
    }
    /// 查找主题的所有订阅者
    pub async fn find_subscribers(&self, topic: &str) -> Vec<TopicSubscription> {
        let root_guard = self.root.read().await;
        let root = &*root_guard;
        let mut subscribers = Vec::new();
        let parts: Vec<&str> = topic.split('.').collect();

        // 使用队列来模拟递归调用
        let mut queue = Vec::new();
        queue.push((root, 0));

        while let Some((node, index)) = queue.pop() {
            if index == parts.len() {
                // 到达主题末尾，添加当前节点的订阅者
                subscribers.extend_from_slice(&node.subscribers);
                continue;
            }

            let current_part = parts[index];

            // 处理精确匹配
            if let Some(child) = node.children.get(current_part) {
                queue.push((child, index + 1));
            }

            // 处理单级通配符 +
            if let Some(child) = node.children.get("+") {
                queue.push((child, index + 1));
            }

            // 处理多级通配符 #
            if let Some(child) = node.children.get("#") {
                subscribers.extend_from_slice(&child.subscribers);
            }
        }

        // 去重，确保每个客户端只返回一个订阅（使用最高的QoS）
        self.deduplicate_subscribers(&mut subscribers).await;

        subscribers
    }

    /// 去重订阅者，保留最高的QoS
    async fn deduplicate_subscribers(&self, subscribers: &mut Vec<TopicSubscription>) {
        let mut unique_subscribers = HashMap::new();

        for sub in subscribers.drain(..) {
            unique_subscribers
                .entry(sub.client_id.clone())
                .and_modify(|existing: &mut TopicSubscription| {
                    // 如果新的QoS更高，更新
                    if sub.qos > existing.qos {
                        existing.qos = sub.qos;
                    }
                })
                .or_insert(sub);
        }

        *subscribers = unique_subscribers.into_values().collect();
    }

    /// 验证主题过滤器是否有效
    pub fn is_valid_topic_filter(&self, topic_filter: &str) -> bool {
        let parts: Vec<&str> = topic_filter.split('.').collect();

        // 检查多级通配符 # 是否只出现在最后
        if let Some((index, part)) = parts.iter().enumerate().find(|&(_, &p)| p == "#") {
            if index != parts.len() - 1 {
                return false;
            }
        }

        // 检查主题过滤器是否包含无效字符
        // MQTT规范规定主题名不能包含空字符
        !topic_filter.contains('\0')
    }

    /// 验证主题名是否有效
    pub fn is_valid_topic(&self, topic: &str) -> bool {
        // MQTT规范规定主题名不能包含空字符，也不能使用通配符
        !topic.contains('\0') && !topic.contains('+') && !topic.contains('#')
    }
}
