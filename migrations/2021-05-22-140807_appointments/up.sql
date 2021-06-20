CREATE TABLE appointments (
    username CHAR(32) NOT NULL,
    tid BIGINT UNSIGNED NOT NULL,
    status CHAR(16) NOT NULL,
    time DATETIME,
    PRIMARY KEY (username, tid)
);

-- hack for timezone offset
CREATE TRIGGER appointments_time_trigger BEFORE INSERT ON appointments FOR EACH ROW BEGIN
    SET NEW.time = CURRENT_TIMESTAMP + INTERVAL 8 HOUR;
END