-- Your SQL goes here
CREATE TABLE IF NOT EXISTS messages (
  id UUID NOT NULL  DEFAULT uuid_generate_v4(),
  sender varchar(30) NOT NULL,
  recipient varchar(30) NOT NULL,
  sent_at timestamp NOT NULL,
  content varchar(1024) NOT NULL,
  content_self_encrypted varchar(1024) NOT NULL,
  PRIMARY KEY(id),
  CONSTRAINT fk_sender FOREIGN KEY(sender) REFERENCES users(username) ON DELETE CASCADE,
  CONSTRAINT fk_recipient FOREIGN KEY(recipient) REFERENCES users(username) ON DELETE CASCADE
)