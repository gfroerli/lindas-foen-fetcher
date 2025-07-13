//! Database module for tracking sent measurements

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use tracing::debug;

/// Create the sent_measurements table
fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sent_measurements (
            sensor_id INTEGER NOT NULL,
            measurement_timestamp INTEGER NOT NULL,
            sent_at INTEGER NOT NULL,
            PRIMARY KEY (sensor_id, measurement_timestamp)
        )",
        [],
    )
    .with_context(|| "Failed to create sent_measurements table")?;
    Ok(())
}

/// Initialize the SQLite database and create the table if it doesn't exist
pub fn init_database(db_path: &str) -> Result<Connection> {
    debug!("Initializing database at {}", db_path);

    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open database at {db_path}"))?;

    create_table(&conn)?;

    debug!("Database initialized successfully");
    Ok(conn)
}

/// Check if a measurement has already been sent for the given sensor and timestamp
pub fn is_measurement_sent(
    conn: &Connection,
    sensor_id: u32,
    measurement_time: &DateTime<Utc>,
) -> Result<bool> {
    let measurement_timestamp = measurement_time.timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT 1 FROM sent_measurements WHERE sensor_id = ? AND measurement_timestamp = ?",
        )
        .with_context(|| "Failed to prepare select statement")?;

    let exists = stmt
        .query_row(params![sensor_id, measurement_timestamp], |_| Ok(()))
        .is_ok();

    Ok(exists)
}

/// Record that a measurement has been successfully sent
pub fn record_measurement_sent(
    conn: &Connection,
    sensor_id: u32,
    measurement_time: &DateTime<Utc>,
) -> Result<()> {
    let measurement_timestamp = measurement_time.timestamp();
    let sent_at = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO sent_measurements (sensor_id, measurement_timestamp, sent_at) VALUES (?, ?, ?)",
        params![sensor_id, measurement_timestamp, sent_at],
    )
    .with_context(|| {
        format!(
            "Failed to record sent measurement for sensor {sensor_id} at timestamp {measurement_timestamp}"
        )
    })?;

    debug!(
        "Recorded sent measurement for sensor {} at timestamp {}",
        sensor_id, measurement_timestamp
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let conn = Connection::open_in_memory().unwrap();

        // Initialize schema
        create_table(&conn).unwrap();

        let test_time = Utc.with_ymd_and_hms(2025, 1, 15, 12, 30, 0).unwrap();
        let sensor_id = 1;

        // Initially, measurement should not be sent
        assert!(!is_measurement_sent(&conn, sensor_id, &test_time).unwrap());

        // Record the measurement as sent
        record_measurement_sent(&conn, sensor_id, &test_time).unwrap();

        // Now it should be detected as already sent
        assert!(is_measurement_sent(&conn, sensor_id, &test_time).unwrap());

        // Different sensor should not be affected
        assert!(!is_measurement_sent(&conn, 2, &test_time).unwrap());

        // Different timestamp should not be affected
        let different_time = Utc.with_ymd_and_hms(2025, 1, 15, 13, 30, 0).unwrap();
        assert!(!is_measurement_sent(&conn, sensor_id, &different_time).unwrap());
    }

    #[test]
    fn test_multiple_sensors_and_timestamps() {
        let conn = Connection::open_in_memory().unwrap();

        // Initialize schema
        create_table(&conn).unwrap();

        let time1 = Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 1, 15, 13, 0, 0).unwrap();

        // Record measurements for different sensors and times
        record_measurement_sent(&conn, 1, &time1).unwrap();
        record_measurement_sent(&conn, 1, &time2).unwrap();
        record_measurement_sent(&conn, 2, &time1).unwrap();

        // Verify all combinations
        assert!(is_measurement_sent(&conn, 1, &time1).unwrap());
        assert!(is_measurement_sent(&conn, 1, &time2).unwrap());
        assert!(is_measurement_sent(&conn, 2, &time1).unwrap());
        assert!(!is_measurement_sent(&conn, 2, &time2).unwrap());
    }
}
