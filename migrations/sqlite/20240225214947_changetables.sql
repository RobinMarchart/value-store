-- Add migration script here

CREATE TABLE changes(
    hash BLOB NOT NULL UNIQUE,
    content BLOB NOT NULL,
    id INT PRIMARY KEY NOT NULL
) STRICT;

CREATE TABLE change_rels(
    parent INT NOT NULL REFERENCES changes (id),
    child INT NOT NULL REFERENCES changes (id),
    id INT PRIMARY KEY NOT NULL,
        CONSTRAINT uniqueness UNIQUE (parent,child)
) STRICT;
