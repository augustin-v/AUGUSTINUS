use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection, OpenFlags};

const MIGRATION_001: &str = include_str!("../migrations/001_init.sql");

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create db dir {}", parent.display()))?;
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("open sqlite db {}", path.display()))?;

        let store = Self { conn };
        store.configure_on_disk()?;
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory sqlite")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn insert_event(&self, kind: &str, payload_json: &str) -> Result<()> {
        let ts = chrono::Utc::now().timestamp();
        self.conn
            .execute(
                "INSERT INTO events(ts, kind, payload_json) VALUES (?1, ?2, ?3)",
                params![ts, kind, payload_json],
            )
            .context("insert event")?;
        Ok(())
    }

    pub fn count_events(&self) -> Result<i64> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .context("count events")?;
        Ok(n)
    }

    pub fn add_focus_seconds_today(&self, seconds: i64) -> Result<()> {
        let today = Local::now().date_naive();
        self.add_focus_seconds_for_day(today, seconds)
    }

    pub fn add_focus_seconds_for_day(&self, day: NaiveDate, seconds: i64) -> Result<()> {
        let day = day.format("%F").to_string();
        self.conn
            .execute(
                r#"
INSERT INTO daily(day, focus_seconds, streak_count, loc_added, loc_removed, calories, lock_in)
VALUES (?1, ?2, 0, 0, 0, 0, 0)
ON CONFLICT(day) DO UPDATE SET focus_seconds = focus_seconds + excluded.focus_seconds
"#,
                params![day, seconds],
            )
            .context("upsert daily focus_seconds")?;
        Ok(())
    }

    pub fn focus_seconds_for_day(&self, day: NaiveDate) -> Result<i64> {
        let day = day.format("%F").to_string();
        let seconds: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(focus_seconds, 0) FROM daily WHERE day = ?1",
                params![day],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(seconds)
    }

    pub fn streak_days_ending_today(&self) -> Result<u32> {
        let mut day = Local::now().date_naive();
        let mut streak: u32 = 0;
        loop {
            let seconds = self.focus_seconds_for_day(day)?;
            if seconds <= 0 {
                break;
            }
            streak += 1;
            if let Some(prev) = day.pred_opt() {
                day = prev;
            } else {
                break;
            }
        }
        Ok(streak)
    }

    pub fn default_db_path() -> Result<PathBuf> {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(Path::new(&xdg).join("augustinus").join("augustinus.db"));
        }
        let home = std::env::var_os("HOME").context("HOME not set")?;
        Ok(Path::new(&home)
            .join(".local")
            .join("share")
            .join("augustinus")
            .join("augustinus.db"))
    }

    fn configure_on_disk(&self) -> Result<()> {
        let _ = self.conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;");
        Ok(())
    }

    fn migrate(&self) -> Result<()> {
        self.conn
            .execute_batch(MIGRATION_001)
            .context("apply migrations")?;
        Ok(())
    }
}

