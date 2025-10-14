//! Database schema migrations
//!
//! This module handles upgrading the database schema when the application
//! is updated to a new version.

use crate::Result;
use rusqlite::Connection;

/// A database migration
pub struct Migration {
    /// Target version
    pub version: i32,

    /// Description
    pub description: &'static str,

    /// Migration function
    pub migrate: fn(&Connection) -> Result<()>,
}

/// All migrations in order
pub static MIGRATIONS: &[Migration] = &[
    // Migration {
    //     version: 2,
    //     description: "Add example_field to contacts table",
    //     migrate: migrate_v2,
    // },
];

/// Run all pending migrations
pub fn run_migrations(conn: &Connection, current_version: i32) -> Result<()> {
    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing::info!(
                "Running migration v{}: {}",
                migration.version,
                migration.description
            );

            (migration.migrate)(conn)?;

            // Update schema version
            conn.execute(
                "UPDATE metadata SET value = ? WHERE key = 'schema_version'",
                [migration.version.to_string()],
            )?;
        }
    }

    Ok(())
}

// Example migration function (for reference)
// fn migrate_v2(conn: &Connection) -> Result<()> {
//     conn.execute(
//         "ALTER TABLE contacts ADD COLUMN example_field TEXT",
//         [],
//     )?;
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_empty() {
        // With no migrations defined, this should be a no-op
        assert_eq!(MIGRATIONS.len(), 0);
    }
}
