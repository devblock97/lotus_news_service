-- Add migration script here
CREATE TABLE votes (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    value SMALLINT NOT NULL CHECK (value IN(-1, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(user_id, post_id)
);

CREATE INDEX votes_post_id_idx ON votes (post_id);