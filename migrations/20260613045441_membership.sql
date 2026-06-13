-- Add migration script here
CREATE TABLE memberships (
    id TEXT PRIMARY KEY NOT NULL,
    community TEXT NOT NULL,
    member TEXT NOT NULL,
    created TEXT NOT NULL,

    FOREIGN KEY (community)
        REFERENCES communities(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_memberships_community
    ON memberships(community);

CREATE INDEX idx_memberships_member
    ON memberships(member);

CREATE INDEX idx_memberships_created
    ON memberships(created);

