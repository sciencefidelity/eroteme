CREATE TABLE IF NOT EXISTS questions (
  id serial PRIMARY KEY,
  title VARCHAR (255) NOT NULL,
  content TEXT [],
  created_on TIMESTAMP NOT NULL DEFAULT NOW()
)
