-- Alternative for NOT NULL
ALTER TABLE posts
ADD COLUMN short_description TEXT NOT NULL DEFAULT '';