CREATE TABLE comments (
    cid SERIAL,
    username CHAR(32) NOT NULL,
    did CHAR(32) NOT NULL,
    comment VARCHAR(256) NOT NULL,
    time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (did)
);