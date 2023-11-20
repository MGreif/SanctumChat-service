-- Your SQL goes here
CREATE TABLE IF NOT EXISTS messages (
  id UUID NOT NULL,
  sender UUID NOT NULL,
  recipient UUID NOT NULL,
  sent_at DATE NOT NULL,
  PRIMARY KEY(id),
  CONSTRAINT fk_sender FOREIGN KEY(sender) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_recipient FOREIGN KEY(recipient) REFERENCES users(id) ON DELETE CASCADE
)