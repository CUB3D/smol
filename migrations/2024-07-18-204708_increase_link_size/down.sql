-- This file should undo anything in `up.sql`

ALTER table links MODIFY COLUMN original_link VARCHAR(128) NOT NULL;