-- Editorial placements have no expiry and are independent of paid campaigns.
CREATE TABLE permanent_features (project_slug TEXT PRIMARY KEY NOT NULL);
INSERT INTO permanent_features (project_slug) VALUES ('shiprust');
