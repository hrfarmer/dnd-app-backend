-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username varchar(32) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    refreshed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE session (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    discord_id INTEGER NOT NULL,
    username varchar(32) NOT NULL,
    discriminator varchar(5) NOT NULL,
    global_name varchar(32) NOT NULL,
    avatar TEXT DEFAULT '' NOT NULL,
    accent_color INTEGER,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
