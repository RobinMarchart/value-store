-- Add migration script here

CREATE TABLE repositories(
    uuid BLOB NOT NULL UNIQUE,
    descr TEXT NOT NULL,
    id INT PRIMARY KEY NOT NULL
) STRICT;

CREATE TABLE branch(
    uuid BLOB NOT NULL,
    repo INT NOT NULL REFERENCES repositories (id),
    head INT NOT NULL REFERENCES changes (id),
    descr TEXT NOT NULL,
    id INT PRIMARY KEY NOT NULL,
       CONSTRAINT uniqueness UNIQUE (uuid,repo)
) STRICT;
