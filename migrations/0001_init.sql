-- swaps: one row per DEX swap (raw in Phase 2, decoded columns filled in Phase 5)
CREATE TABLE swaps (
    id             BIGSERIAL PRIMARY KEY,
    signature      TEXT        NOT NULL,
    slot           BIGINT      NOT NULL,
    block_time     TIMESTAMPTZ,
    program        TEXT        NOT NULL,
    dex            TEXT        NOT NULL DEFAULT 'unknown',
    wallet         TEXT,
    token_in_mint  TEXT,
    token_out_mint TEXT,
    amount_in      NUMERIC,
    amount_out     NUMERIC,
    decoded        BOOLEAN     NOT NULL DEFAULT false,
    raw            JSONB       NOT NULL,
    indexed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (signature, program)
);

CREATE INDEX idx_swaps_block_time ON swaps (block_time DESC);
CREATE INDEX idx_swaps_slot       ON swaps (slot DESC);
CREATE INDEX idx_swaps_dex        ON swaps (dex);
CREATE INDEX idx_swaps_wallet     ON swaps (wallet);
CREATE INDEX idx_swaps_mints      ON swaps (token_in_mint, token_out_mint);

-- dlq: rows that failed to parse/ingest, so nothing is silently lost
CREATE TABLE dlq (
    id         BIGSERIAL PRIMARY KEY,
    signature  TEXT,
    program    TEXT,
    stage      TEXT        NOT NULL,
    error      TEXT        NOT NULL,
    raw        JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- indexer_state: single row (id=1) for resume-from-slot
CREATE TABLE indexer_state (
    id         SMALLINT    PRIMARY KEY DEFAULT 1,
    last_slot  BIGINT      NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT single_row CHECK (id = 1)
);

INSERT INTO indexer_state (id, last_slot) VALUES (1, 0);
