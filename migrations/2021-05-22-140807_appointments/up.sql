CREATE TABLE appointments (
    username CHAR(32) NOT NULL,
    tid BIGINT UNSIGNED NOT NULL,
    status CHAR(16) NOT NULL,
    time DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (username, tid)
)