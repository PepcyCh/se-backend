CREATE TABLE doctors (
    did CHAR(32) NOT NULL,
    name CHAR(10) NOT NULL,
    password CHAR(128) NOT NULL,
    gender CHAR(10) NOT NULL,
    birthday DATE,
    department CHAR(32) NOT NULL,
    rankk CHAR(32) NOT NULL,
    information VARCHAR(256) NOT NULL,
    PRIMARY KEY (did)
)