CREATE TABLE doctor_logins (
    token CHAR(128) NOT NULL,
    did CHAR(32) NOT NULL,
    login_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (token, did, login_time)
);