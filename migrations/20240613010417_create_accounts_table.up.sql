CREATE TABLE IF NOT EXISTS accounts (
  id serial NOT NULL,
  email VARCHAR(255) NOT NULL PRIMARY KEY,
  password VARCHAR(255) NOT NULL
);