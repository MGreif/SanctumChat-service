-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  id UUID NOT NULL  DEFAULT uuid_generate_v4() ,
  name varchar(30) NOT NULL,
  age int NOT NULL,
  password varchar(64) NOT NULL,
  PRIMARY KEY(id)
)