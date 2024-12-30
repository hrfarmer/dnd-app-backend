-- Add migration script here
ALTER TABLE campaign_invites ALTER COLUMN uses SET NOT NULL;
