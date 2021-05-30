CREATE TABLE user_logins (
    token CHAR(128) NOT NULL,
    username CHAR(32) NOT NULL,
    login_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (token, username, login_time)
);