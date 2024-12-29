-- Add migration script here
CREATE TABLE dnd_session (
	id SERIAL PRIMARY KEY,
	creator_id INTEGER NOT NULL,
	name varchar(256) NOT NULL,
	created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

	CONSTRAINT fk_user FOREIGN KEY (creator_id) REFERENCES users (id) ON DELETE CASCADE
)
