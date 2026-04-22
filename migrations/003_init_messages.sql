-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    sender_type SMALLINT NOT NULL DEFAULT 0,  -- 0=user, 1=agent, 2=system
    sender_id   UUID,                          -- users.id or agents.id
    content     TEXT NOT NULL,
    content_type VARCHAR(16) NOT NULL DEFAULT 'text',  -- text, markdown, code, image_url
    token_count INTEGER DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_messages_session ON messages(session_id, created_at);
CREATE INDEX idx_messages_created ON messages(created_at);

-- Tasks extracted from messages (for tracking agent work items)
CREATE TABLE IF NOT EXISTS tasks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    assigned_to UUID REFERENCES agents(id),
    title       VARCHAR(255) NOT NULL,
    description TEXT,
    status      VARCHAR(16) NOT NULL DEFAULT 'pending',  -- pending, in_progress, done, failed
    priority    SMALLINT NOT NULL DEFAULT 0,
    due_at      TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tasks_session ON tasks(session_id);
CREATE INDEX idx_tasks_status ON tasks(status);
