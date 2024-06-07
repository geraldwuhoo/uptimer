-- Add up migration script here

DROP TABLE IF EXISTS site;
CREATE TABLE site (
    site text NOT NULL PRIMARY KEY,
    name text NOT NULL
);
