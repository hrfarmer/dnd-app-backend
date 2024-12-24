-- Add migration script here
ALTER TABLE session
ALTER COLUMN discord_id TYPE TEXT;
