//! Display and output formatting functions

use crate::parsing::StationMeasurement;

/// Prints the table header for measurement results
pub fn print_table_header() {
    println!("\nResults:");
    println!(
        "{:<10} {:<30} {:<25} {:<15}",
        "Station ID", "Station Name", "Time", "Temperature (°C)"
    );
    println!("{}", "-".repeat(85));
}

/// Prints a single measurement row
pub fn print_measurement_row(measurement: &StationMeasurement) {
    let formatted_time = measurement.time.format("%Y-%m-%d %H:%M:%S %z").to_string();
    let formatted_temperature = format!("{:.3}", measurement.temperature);

    println!(
        "{:<10} {:<30} {:<25} {:<15}",
        measurement.station_id, measurement.station_name, formatted_time, formatted_temperature
    );
}

/// Prints a message when no data is found for a station
pub fn print_no_data_message(station_id: u32) {
    println!("{station_id:<10} No temperature data found");
}

/// Prints the final summary
pub fn print_summary(total_records: usize) {
    println!("\nTotal records found: {total_records}");
}

/// Prints error summary if there were any errors during data fetching
pub fn print_error_summary(error_count: usize) {
    if error_count > 0 {
        eprintln!("\nWarning: {error_count} station(s) had errors during data fetching");
    }
}
