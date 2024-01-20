-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  username varchar(30) NOT NULL,
  password varchar(64) NOT NULL,
  public_key BYTEA NOT NULL,
  PRIMARY KEY(username)
);

CREATE TABLE IF NOT EXISTS messages (
  id UUID NOT NULL  DEFAULT uuid_generate_v4(),
  sender varchar(30) NOT NULL,
  recipient varchar(30) NOT NULL,
  sent_at timestamp NOT NULL,
  content varchar(1024) NOT NULL,
  content_self_encrypted varchar(1024) NOT NULL,
  content_signature varchar(1024) NOT NULL,
  content_self_encrypted_signature varchar(1024) NOT NULL,
  PRIMARY KEY(id),
  CONSTRAINT fk_sender FOREIGN KEY(sender) REFERENCES users(username) ON DELETE CASCADE,
  CONSTRAINT fk_recipient FOREIGN KEY(recipient) REFERENCES users(username) ON DELETE CASCADE
);


-- Friends

CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    sender varchar(30) NOT NULL,
    recipient varchar(30) NOT NULL,
    accepted bool, -- Can be null if friend request is pending
    CONSTRAINT fk_sender FOREIGN KEY(sender) REFERENCES users(username),
    CONSTRAINT fk_recipient FOREIGN KEY(recipient) REFERENCES users(username),
    PRIMARY KEY(id) 
);


CREATE TABLE IF NOT EXISTS friends (
	id UUID NOT NULL DEFAULT uuid_generate_v4(),
	PRIMARY KEY(id),
	user_id varchar(30) NOT NULL,
	befriended_user_id varchar(30) NOT NULL,
	CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(username),
	CONSTRAINT fk_befriended_user_id FOREIGN KEY(befriended_user_id) REFERENCES users(username)
);

CREATE OR REPLACE FUNCTION add_friends() RETURNS trigger AS $add_friends$
    BEGIN
        IF NEW.accepted IS true THEN
            INSERT INTO friends(user_id, befriended_user_id) values(NEW.sender, NEW.recipient), (NEW.recipient, NEW.sender);
        END IF;
        RETURN NULL;
    END;
$add_friends$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER add_friends AFTER UPDATE ON friend_requests
    FOR EACH ROW EXECUTE FUNCTION add_friends();