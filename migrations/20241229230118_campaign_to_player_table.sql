-- Add migration script here
CREATE TABLE campaign_players (
	campaign_id INTEGER REFERENCES campaign(id),
	player_id INTEGER REFERENCES users(id),
	joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	role VARCHAR(25) DEFAULT 'player',
	PRIMARY KEY (campaign_id, player_id)
);
