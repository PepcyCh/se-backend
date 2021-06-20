CREATE TABLE comments (
    cid SERIAL,
    username CHAR(32) NOT NULL,
    did CHAR(32) NOT NULL,
    comment VARCHAR(256) NOT NULL,
    time DATETIME,
    PRIMARY KEY (did)
);

-- hack for timezone offset
CREATE TRIGGER comments_time_trigger BEFORE INSERT ON comments FOR EACH ROW BEGIN
    SET NEW.time = CURRENT_TIMESTAMP + INTERVAL 8 HOUR;
END