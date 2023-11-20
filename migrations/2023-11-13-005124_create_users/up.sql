-- Your SQL goes here
CREATE TABLE users (
  name varchar(30) NOT NULL,
  age int NOT NULL,
  id UUID NOT NULL,
  password varchar(64) NOT NULL,
  PRIMARY KEY(id)
)