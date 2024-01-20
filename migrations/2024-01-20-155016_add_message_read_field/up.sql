-- Your SQL goes here
ALTER TABLE messages
ADD COLUMN is_read BOOLEAN NOT NULL DEFAULT false;
