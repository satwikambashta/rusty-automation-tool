-- Migration: 001 — Create initial schema
-- Idempotent via IF NOT EXISTS / DO NOTHING patterns.

-- Enable pgcrypto for gen_random_uuid() if not already enabled.
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- workflows
-- ============================================================
CREATE TABLE IF NOT EXISTS workflows (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT        NOT NULL,
    definition JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflows_created_at ON workflows (created_at DESC);

-- ============================================================
-- workflow_executions
-- ============================================================
CREATE TABLE IF NOT EXISTS workflow_executions (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID        NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status      TEXT        NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending', 'running', 'succeeded', 'failed')),
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wexec_workflow_id ON workflow_executions (workflow_id);
CREATE INDEX IF NOT EXISTS idx_wexec_status      ON workflow_executions (status);

-- ============================================================
-- node_executions
-- ============================================================
CREATE TABLE IF NOT EXISTS node_executions (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID        NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    node_id      TEXT        NOT NULL,
    input        JSONB       NOT NULL,
    output       JSONB,
    status       TEXT        NOT NULL DEFAULT 'pending'
                             CHECK (status IN ('pending', 'running', 'succeeded', 'failed')),
    started_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_nexec_execution_id ON node_executions (execution_id);

-- ============================================================
-- secrets
-- ============================================================
CREATE TABLE IF NOT EXISTS secrets (
    id              UUID  PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id     UUID  NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    key             TEXT  NOT NULL,
    encrypted_value TEXT  NOT NULL,
    UNIQUE (workflow_id, key)
);

CREATE INDEX IF NOT EXISTS idx_secrets_workflow_id ON secrets (workflow_id);

-- ============================================================
-- job_queue
-- ============================================================
CREATE TABLE IF NOT EXISTS job_queue (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID        NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    workflow_id  UUID        NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    status       TEXT        NOT NULL DEFAULT 'pending'
                             CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'dead_lettered')),
    attempts     INT         NOT NULL DEFAULT 0,
    max_attempts INT         NOT NULL DEFAULT 3,
    payload      JSONB       NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_job_queue_status     ON job_queue (status, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_job_queue_exec_id    ON job_queue (execution_id);
