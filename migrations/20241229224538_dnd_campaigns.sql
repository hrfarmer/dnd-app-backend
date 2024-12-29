-- Add migration script here
CREATE TABLE campaign (
	id SERIAL PRIMARY KEY,
	user_id INTEGER NOT NULL,
	name varchar(128) NOT NULL,
	image_link TEXT,
	created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

	CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE RESTRICT
);

ALTER TABLE dnd_session
    RENAME COLUMN creator_id TO user_id;

ALTER TABLE dnd_session
    ADD COLUMN campaign_id INTEGER NOT NULL;

ALTER TABLE dnd_session
    ADD CONSTRAINT fk_campaign FOREIGN KEY (campaign_id) REFERENCES campaign (id) ON DELETE CASCADE;
