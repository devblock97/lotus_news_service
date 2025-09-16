-- Add migration script here
CREATE TABLE posts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    url TEXT,
    body TEXT,
    score INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (url IS NOT NULL AND body IS NULL)
        OR (url IS NULL AND body IS NOT NULL)
        OR (url IS NULL AND body IS NULL) -- allow placeholder during drafts if you want
    )
);

CREATE INDEX posts_user_id_idx ON posts (user_id);
CREATE INDEX posts_created_at_idx ON posts (created_at);
CREATE INDEX posts_score_idx ON posts (score);