-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    age INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sample data
INSERT OR IGNORE INTO users (name, email, age) VALUES 
    ('Alice Johnson', 'alice@example.com', 28),
    ('Bob Smith', 'bob@example.com', 34),
    ('Carol Davis', 'carol@example.com', 26),
    ('David Wilson', 'david@example.com', 31),
    ('Eva Brown', 'eva@example.com', 29);

-- User preferences table for recommendations
CREATE TABLE IF NOT EXISTS user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    preference_score REAL DEFAULT 0.5,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Sample preferences
INSERT OR IGNORE INTO user_preferences (user_id, category, preference_score) VALUES 
    (1, 'technology', 0.9),
    (1, 'sports', 0.3),
    (1, 'music', 0.7),
    (2, 'technology', 0.4),
    (2, 'sports', 0.8),
    (2, 'cooking', 0.6),
    (3, 'music', 0.9),
    (3, 'travel', 0.8),
    (3, 'books', 0.7);

-- User interactions for ML training
CREATE TABLE IF NOT EXISTS user_interactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    interaction_type TEXT NOT NULL, -- view, like, share, purchase
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Sample interactions
INSERT OR IGNORE INTO user_interactions (user_id, item_type, item_id, interaction_type) VALUES 
    (1, 'article', 'tech-001', 'view'),
    (1, 'article', 'tech-001', 'like'),
    (1, 'product', 'gadget-123', 'view'),
    (2, 'article', 'sports-005', 'view'),
    (2, 'article', 'sports-005', 'share'),
    (3, 'song', 'song-456', 'view'),
    (3, 'song', 'song-456', 'like'),
    (3, 'playlist', 'indie-rock-2023', 'view');

-- Analytics tracking
CREATE TABLE IF NOT EXISTS analytics_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    event_type TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    response_time INTEGER,
    status_code INTEGER,
    user_agent TEXT,
    ip_address TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_user_interactions_user_id ON user_interactions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_interactions_timestamp ON user_interactions(timestamp);
CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_analytics_events_endpoint ON analytics_events(endpoint);
