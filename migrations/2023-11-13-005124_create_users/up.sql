-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  username varchar(30) NOT NULL,
  name varchar(50) NOT NULL,
  age int NOT NULL,
  password varchar(64) NOT NULL,
  public_key BYTEA NOT NULL,
  PRIMARY KEY(username)
)