CREATE TABLE admin_logins (
    token CHAR(128) NOT NULL,
    aid CHAR(32) NOT NULL,
    login_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (token, aid, login_time)
);