-- Add up migration script here

DROP TABLE IF EXISTS site_fact;
CREATE TABLE site_fact (
    site text NOT NULL,
    tstamp timestamptz NOT NULL,
    success boolean NOT NULL,
    status_code smallint NOT NULL,
    PRIMARY KEY(site, tstamp)
);
