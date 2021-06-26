CREATE TABLE users (
    username CHAR(32) NOT NULL,
    password CHAR(128) NOT NULL,
    name CHAR(32) NOT NULL,
    gender CHAR(10) NOT NULL,
    birthday DATE,
    id_number CHAR(20) NOT NULL,
    telephone CHAR(16) NOT NULL,
    is_banned BOOL NOT NULL,
    PRIMARY KEY (username)
);