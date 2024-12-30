-- Add migration script here

CREATE TABLE campaign_invites (
	campaign_id INTEGER REFERENCES campaign(id),
	invite varchar(24) NOT NULL,
	created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	uses INTEGER DEFAULT 0,

	PRIMARY KEY (campaign_id)
)
