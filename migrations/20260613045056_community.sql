-- Add migration script here
CREATE TABLE communities (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created TEXT NOT NULL
);

CREATE INDEX idx_communities_created
    ON communities(created);
