CREATE TABLE organization (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  login TEXT NOT NULL,
  name TEXT NOT NULL,
  created DATETIME DEFAULT current_timestamp,
  updated DATETIME DEFAULT current_timestamp
);
CREATE TABLE note (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  org_id INTEGER NOT NULL,
  category TEXT,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  edek TEXT NOT NULL,
  created DATETIME DEFAULT current_timestamp,
  updated DATETIME DEFAULT current_timestamp,
  FOREIGN KEY(org_id) REFERENCES organization(id)
);
CREATE TABLE attachment (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER,
  filename TEXT NOT NULL,
  created DATETIME DEFAULT current_timestamp,
  FOREIGN KEY(note_id) REFERENCES note(id)
);
