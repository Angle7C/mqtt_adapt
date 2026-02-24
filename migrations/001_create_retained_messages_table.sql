-- 创建保留消息表
CREATE TABLE IF NOT EXISTS retained_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL UNIQUE,
    payload BLOB NOT NULL,
    qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建主题索引
CREATE INDEX IF NOT EXISTS idx_retained_messages_topic ON retained_messages(topic);
