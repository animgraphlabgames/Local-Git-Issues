use crate::models::Label;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS labels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL,
            description TEXT DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS issues (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS issue_labels (
            issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            label_id INTEGER NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
            PRIMARY KEY (issue_id, label_id)
        );
        CREATE TABLE IF NOT EXISTS issue_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            event_type TEXT NOT NULL,
            old_value TEXT,
            new_value TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS issue_revisions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    )?;

    let _ = conn.execute("ALTER TABLE issues ADD COLUMN project_id INTEGER", []);

    let label_count: i64 = conn.query_row("SELECT COUNT(*) FROM labels", [], |r| r.get(0))?;
    if label_count == 0 {
        conn.execute(
            "INSERT INTO labels (name, color, description) VALUES
            ('bug', '#d73a4a', 'Something isn''t working'),
            ('enhancement', '#a2eeef', 'New feature or request'),
            ('documentation', '#0075ca', 'Improvements or additions to documentation'),
            ('topic:editor', '#fbca04', 'Editor subsystem'),
            ('platform:macos', '#c5def5', 'macOS specific'),
            ('regression', '#e99695', 'Regression from previous version')",
            [],
        )?;
    }

    let project_count: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
    if project_count == 0 {
        conn.execute(
            "INSERT INTO projects (title, description) VALUES 
            ('Desktop Platform Support', 'macOS, Windows, and Linux windowing fixes')",
            [],
        )?;
    }

    let issue_count: i64 = conn.query_row("SELECT COUNT(*) FROM issues", [], |r| r.get(0))?;
    if issue_count == 0 {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO issues (title, body, status, project_id, created_at) VALUES 
            (?1, ?2, 'open', 1, ?3),
            (?4, ?5, 'closed', 1, ?3)",
            params![
                "macOS: every mouse button press suppresses mouse motion events for 150-250 ms",
                "### Tested versions\nReproducible in 4.6.1.stable.mono (official, 14d1969).\n\n### System information\nmacOS 26.6.2 (25G83), Apple M1 Max, Forward+ / Metal\n\n### Issue description\n> On macOS, every mouse button press stops `InputEventMouseMotion` delivery for roughly 150 to 250 ms.\n\n```js\nvar lol = 12;\nif (true) {\n  console.log('panning');\n}\n```\n\n| duration | motion events | clicks |\n| :--- | :--- | :--- |\n| 26.8 s | 65.1/s | 1.7/s |\n| 20.8 s | 109.8/s | 2.8/s |",
                now,
                "Mouse capture does not work on Embedded Game windows on Wayland",
                "Wayland embedded display surface fails to bind absolute positioning constraints properly.",
            ],
        )?;

        let _ = conn.execute("INSERT INTO issue_labels (issue_id, label_id) VALUES (1, 1), (1, 4), (1, 5), (2, 1)", []);
        let _ = conn.execute("INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (1, ?1, ?2)", params![
            "### Tested versions\nReproducible in 4.6.1.stable.mono (official, 14d1969).\n\n### System information\nmacOS 26.6.2 (25G83), Apple M1 Max, Forward+ / Metal\n\n### Issue description\n> On macOS, every mouse button press stops `InputEventMouseMotion` delivery for roughly 150 to 250 ms.\n\n```js\nvar lol = 12;\nif (true) {\n  console.log('panning');\n}\n```\n\n| duration | motion events | clicks |\n| :--- | :--- | :--- |\n| 26.8 s | 65.1/s | 1.7/s |\n| 20.8 s | 109.8/s | 2.8/s |",
            now
        ]);
        let _ = conn.execute("INSERT INTO issue_revisions (issue_id, body, created_at) VALUES (2, ?1, ?2)", params![
            "Wayland embedded display surface fails to bind absolute positioning constraints properly.",
            now
        ]);
    }
    Ok(())
}

pub fn fetch_labels_for_issue(conn: &Connection, issue_id: i64) -> rusqlite::Result<Vec<Label>> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.name, l.color, l.description 
         FROM labels l 
         JOIN issue_labels il ON l.id = il.label_id 
         WHERE il.issue_id = ?1 
         ORDER BY l.name ASC",
    )?;
    let iter = stmt.query_map(params![issue_id], |row| {
        Ok(Label {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            description: row.get(3)?,
        })
    })?;
    let mut labels = Vec::new();
    for l in iter {
        labels.push(l?);
    }
    Ok(labels)
}