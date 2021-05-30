CREATE TABLE times (
    tid SERIAL,
    did CHAR(32) NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    capacity INT NOT NULL,
    rest INT NOT NULL DEFAULT 0,
    PRIMARY KEY (tid)
);