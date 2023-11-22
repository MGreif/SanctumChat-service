-- Your SQL goes here
CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    sender UUID NOT NULL,
    recipient UUID NOT NULL,
    accepted bool, -- Can be null if friend request is pending
    CONSTRAINT fk_sender FOREIGN KEY(sender) REFERENCES users(id),
    CONSTRAINT fk_recipient FOREIGN KEY(recipient) REFERENCES users(id),
    PRIMARY KEY(id) 
);


CREATE TABLE IF NOT EXISTS friends (
	id UUID NOT NULL DEFAULT uuid_generate_v4(),
	PRIMARY KEY(id),
	user_id UUID NOT NULL,
	befriended_user_id UUID NOT NULL,
	CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id),
	CONSTRAINT fk_befriended_user_id FOREIGN KEY(befriended_user_id) REFERENCES users(id)
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