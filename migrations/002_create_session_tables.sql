-- 创建会话表
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL UNIQUE,
    clean_session BOOLEAN NOT NULL DEFAULT 0,
    connected BOOLEAN NOT NULL DEFAULT 0,
    last_connected_at DATETIME,
    last_disconnected_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建会话订阅表
CREATE TABLE IF NOT EXISTS session_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    topic TEXT NOT NULL,
    qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, topic)
);

-- 创建离线消息表
CREATE TABLE IF NOT EXISTS offline_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    topic TEXT NOT NULL,
    payload BLOB NOT NULL,
    qos INTEGER NOT NULL CHECK (qos IN (0, 1, 2)),
    packet_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_sessions_client_id ON sessions(client_id);
CREATE INDEX IF NOT EXISTS idx_session_subscriptions_session_id ON session_subscriptions(session_id);
CREATE INDEX IF NOT EXISTS idx_offline_messages_session_id ON offline_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_offline_messages_created_at ON offline_messages(created_at);
