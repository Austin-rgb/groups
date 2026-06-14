-- Add migration script here
CREATE TABLE invite (
    id TEXT PRIMARY KEY NOT NULL,
    community TEXT NOT NULL,
    user TEXT NOT NULL,
    created TEXT NOT NULL,
    exp TEXT NOT NULL,

    UNIQUE (community, user)
);
