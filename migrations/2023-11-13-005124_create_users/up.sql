-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  username varchar(30) NOT NULL,
  name varchar(50) NOT NULL,
  age int NOT NULL,
  password varchar(64) NOT NULL,
  PRIMARY KEY(username)
)