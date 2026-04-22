-- Chat sessions (1:1 and 1:N)
CREATE TABLE IF NOT EXISTS sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL DEFAULT 'New Chat',
    session_type SMALLINT NOT NULL DEFAULT 1,  -- 1=1:1, 2=group
    created_by  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Session members (for group chats)
CREATE TABLE IF NOT EXISTS session_members (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role        VARCHAR(32) NOT NULL DEFAULT 'member',  -- owner, admin, member, agent
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(session_id, user_id)
);

CREATE INDEX idx_session_members_session ON session_members(session_id);
CREATE INDEX idx_session_members_user ON session_members(user_id);

-- AI agents registered in the system
CREATE TABLE IF NOT EXISTS agents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(64) NOT NULL UNIQUE,
    role        VARCHAR(32) NOT NULL,  -- pm, dev, qa, approval
    system_prompt TEXT NOT NULL,
    model       VARCHAR(64) NOT NULL DEFAULT 'llama3',
    tools       JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
