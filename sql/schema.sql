-- Phloem SQLite Database Schema

-- Command suggestions cache
CREATE TABLE IF NOT EXISTS suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt_hash TEXT NOT NULL,
    prompt TEXT NOT NULL,
    suggestion TEXT NOT NULL,
    explanation TEXT,
    confidence REAL DEFAULT 0.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    use_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    success_rate REAL DEFAULT 0.5
);

-- Create unique index on prompt_hash + suggestion combination
CREATE UNIQUE INDEX IF NOT EXISTS idx_suggestions_unique ON suggestions(prompt_hash, suggestion);

-- Command execution history
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    prompt TEXT NOT NULL,
    success BOOLEAN DEFAULT TRUE,
    exit_code INTEGER,
    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    context_snapshot TEXT -- JSON of environment at execution time
);

-- Environment tracking
CREATE TABLE IF NOT EXISTS environment (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    detected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_suggestions_prompt_hash ON suggestions(prompt_hash);
CREATE INDEX IF NOT EXISTS idx_suggestions_created_at ON suggestions(created_at);
CREATE INDEX IF NOT EXISTS idx_history_executed_at ON history(executed_at);
CREATE INDEX IF NOT EXISTS idx_environment_updated_at ON environment(updated_at);