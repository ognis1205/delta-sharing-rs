-- Add migration script here
ALTER TABLE token DROP COLUMN email;
ALTER TABLE token DROP COLUMN "role";
ALTER TABLE token ADD COLUMN active BOOLEAN NOT NULL;
ALTER TABLE token ADD COLUMN created_for UUID NOT NULL REFERENCES account(id);
