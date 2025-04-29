use std::fs::File;
use std::path::Path;
use csv::Writer;
use std::time::SystemTime;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Local};


// CPU Temperature Function.
pub fn read_cpu_temperature() -> Option<f32> {
    if let Ok(contents) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(temp) = contents.trim().parse::<f32>() {
            let rounded_temp = (temp / 1000.0 * 100.0).round() / 100.0;
            return Some(rounded_temp);
        }
    }
    None
}

// Logging function.
pub fn log_event(message: &str, log_option: bool) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted_message = format!("[{}] {}", timestamp, message);

    eprintln!("{}", formatted_message);

    if log_option {
        if let Ok(mut file) = OpenOptions::new().append(true).create(true).open("event.log") {
            let _ = writeln!(file, "{}", formatted_message);
        }
    }
}

// Function to log statistics and update values.
pub fn log_statistics
(
// Define input arguments and types.
    i_stat: &mut u64, n_on_temp: &mut f32, n_on_fan: &mut f32,
    stat_min_time: u64, stat_max_time: u64, stat_delta_t: f32, stat_int_start: &mut SystemTime,
    log_option: bool,
    duty_cycle: f32,
    cpu_temp_missing: bool,
    cpu_temp_orig: Option<f32>, cpu_temp_min: &mut Option<f32>, cpu_temp_max: &mut Option<f32>, cpu_temp_avg: &mut Option<f32>,
    fan_on: bool,
    fan_state_stat: &mut f32, fan_state_prev: &mut bool,
    fan_speed_min: &mut Option<f32>, fan_speed_max: &mut Option<f32>, fan_speed_avg: &mut Option<f32>,
    file_path: &str, shdn_recv: bool, rld_recv: bool
) -> (

// Define output types.
    u64, f32, f32, // ........................... i_stat, n_on_temp, n_on_fan
    SystemTime, // .............................. stat_int_start
    Option<f32>, Option<f32>, Option<f32>, // ... cpu_temp_min, cpu_temp_max, cpu_temp_avg
    f32, bool, // ............................... fan_state_stat, fan_state_prev
    Option<f32>, Option<f32>, Option<f32>, // ... fan_speed_min, fan_speed_max, fan_speed_avg
)
// Function.
{

    // Convert the statistics integer to f32 so that it can be used for calculations.
    let n: f32 = *i_stat as f32;


    // If the fan is on, use the duty cycle as the fan speed.
    let fan_speed: Option<f32> = if fan_on { Some(duty_cycle) } else { None };


    // Convert the fan speed to a numerical vale.
    let fan_state_num: f32 = if fan_on { 1.0 } else { 0.0 };

    
    // Check if the fan was either turned on or off.
    let fan_state_chng: bool = fan_on != *fan_state_prev;
    *fan_state_prev = fan_on;


    // Calculate statistics.
    if *i_stat == 1 {

        // Initially only assign the current values.
        // Temperature statistics.
        if !cpu_temp_missing {*n_on_temp += 1.0;}
        *cpu_temp_min = cpu_temp_orig;
        *cpu_temp_max = cpu_temp_orig;
        *cpu_temp_avg = cpu_temp_orig;
        // Fan statistics.
        if fan_on {*n_on_fan += 1.0;}
        *fan_state_stat = fan_state_num;
        *fan_speed_min = fan_speed;
        *fan_speed_max = fan_speed;
        *fan_speed_avg = fan_speed;
        // Start time of statistic period.
        *stat_int_start = SystemTime::now();

    } else {

        // For every iteration after the first calculate statistics based on the statistics values and the current values.
        // Temperature statistics.
        if !cpu_temp_missing {*n_on_temp += 1.0;}
        *cpu_temp_min = min_option(*cpu_temp_min, cpu_temp_orig);
        *cpu_temp_max = max_option(*cpu_temp_max, cpu_temp_orig);
        *cpu_temp_avg = avg_option(*cpu_temp_avg, cpu_temp_orig, *n_on_temp);
        // Fan statistics.
        if fan_on {*n_on_fan += 1.0;}
        *fan_state_stat = (*fan_state_stat * (n - 1.0) / n) + (fan_state_num / n);
        *fan_speed_min = min_option(*fan_speed_min, fan_speed);
        *fan_speed_max = max_option(*fan_speed_max, fan_speed);
        *fan_speed_avg = avg_option(*fan_speed_avg, fan_speed, *n_on_fan);
    }


    // Log and reset values when the condition is met
    // First ensure that the duration since the beginning of this stat period can be determined. If not, skip logging.
    if let Ok(elapsed) = stat_int_start.elapsed() {
    // Log stats if either ...
    if 
    // ... the minimum duration since the last log has past and the observed temperature difference is at least stat_delta_t, ....
    ((elapsed.as_secs() >= stat_min_time && delta_option(*cpu_temp_max, *cpu_temp_min).unwrap_or(0.0) >= stat_delta_t) | 
    // ... the minimum duration since the last log has past and the fan was either turned on or off, ...
    (elapsed.as_secs() >= stat_min_time && fan_state_chng) |
    // ... or the maximum duration since the last log has past.
    (elapsed.as_secs() >= stat_max_time)) | 
    // If a SIGINT/SIGTERM/SIGHUP signal is received, always log stats.
    (shdn_recv | rld_recv)
    {
        // Define time points
        let stat_int_start_convert: DateTime<Local> = DateTime::from(*stat_int_start);
        let timestamp_start = stat_int_start_convert.format("%Y-%m-%d %H:%M:%S").to_string();
        let timestamp_end = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Write the data
        if append_to_csv
        (
            // Input arguments
            log_option,
            file_path,
            timestamp_start, timestamp_end,
            *cpu_temp_min, *cpu_temp_max, *cpu_temp_avg,
            *fan_state_stat,
            *fan_speed_min, *fan_speed_max, *fan_speed_avg
        ) {
            // If statistics where successfully written to the .csv file, reset statistics period.
            *i_stat = 0;
            *n_on_fan = 0.0;
            *n_on_temp = 0.0;
        };
        
        
    } }

    // Return arguments
    (
        *i_stat, *n_on_temp, *n_on_fan,
        *stat_int_start,
        *cpu_temp_min, *cpu_temp_max, *cpu_temp_avg,
        *fan_state_stat, *fan_state_prev,
        *fan_speed_min, *fan_speed_max, *fan_speed_avg
    )
}


// Return the min of two Option<f32> variables.
// If one of them has no value, return the value of the other on. If both have no value, return none.
fn min_option(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if x < y {x} else {x * 0.9 + y * 0.1}),
        // (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y), 
        (None, None) => None
    }
}


// Return the max of two Option<f32> variables.
// If one of them has no value, return the value of the other on. If both have no value, return none.
fn max_option(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if x > y {x} else {x * 0.9 + y * 0.1}),
        // (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y), 
        (None, None) => None,
    }
}


// Extend an Option<f32> average of (n-1) values with another value with the average of all n values as the result.
// If one of them has no value, return the value of the other on. If both have no value, return none.
fn avg_option(a: Option<f32>, b: Option<f32>, c: f32) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some((x * (c - 1.0) / c) + (y / c)), 
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}


// Return the difference between two Option<f32> values. Only return a value if both have some value, else return none.
fn delta_option (a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x - y),
        (Some(_x), None) => None,
        (None, Some(_y)) => None,
        (None, None) => None,
    }
}


// Create the .csv file for the statistics and add the header row.
pub fn create_csv(file_path: &str, log_option: bool) {

    // Check if the file already exists. If it does, do nothing.
    if !Path::new(file_path).exists() {

        // If it does not exist, create the file.
        let file = match File::create(file_path) {
            Ok(f) => f,
            Err(e) => {
                log_event(&format!("Failed to create file {}: {}", file_path, e), log_option);
                return;
            }
        };

        let mut wtr = Writer::from_writer(file);

        // Define the header row.
        if let Err(e) = wtr.write_record(&["start", "end", "cpu_temp_min", "cpu_temp_max", "cpu_temp_avg", "fan_state_stat", "fan_speed_min", "fan_speed_max", "fan_speed_avg"]) {
            log_event(&format!("Failed to write header to CSV: {}", e), log_option);
            return;
        }

        // Write the header row to the file.
        if let Err(e) = wtr.flush() {
            log_event(&format!("Failed to flush CSV writer: {}", e), log_option);
            return;
        } 
    }
}


// Write statistics data to the .csv file.
fn append_to_csv
(
// Define input arguments and types.
    log_option: bool,
    file_path: &str,
    timestamp_start: String, timestamp_end: String,
    cpu_temp_min: Option<f32>, cpu_temp_max: Option<f32>, cpu_temp_avg: Option<f32>,
    fan_state_stat: f32,
    fan_speed_min: Option<f32>, fan_speed_max: Option<f32>, fan_speed_avg: Option<f32> 
) -> 

// Define output types.
bool 

// Function
{
    // Check if the file exists. If not, create it.
    if !Path::new(file_path).exists() {
        create_csv(file_path, log_option)
    };
    
    // Open the file in append mode.
    let file = match OpenOptions::new().append(true).open(file_path) {
        Ok(f) => f,
        Err(e) => {
            log_event(&format!("Failed to open file {}: {}", file_path, e), log_option);
            return false;
        }
    };

    let mut wtr = Writer::from_writer(file);

    // Create the data row from the stats.
    let data_row = vec![
        timestamp_start,
        timestamp_end,
        cpu_temp_min.map_or("NaN".to_string(), |v| format!("{:.3}", v)),
        cpu_temp_max.map_or("NaN".to_string(), |v| format!("{:.3}", v)),
        cpu_temp_avg.map_or("NaN".to_string(), |v| format!("{:.3}", v)),
        format!("{:.3}", fan_state_stat),
        fan_speed_min.map_or("NaN".to_string(), |v| format!("{:.3}", v)),
        fan_speed_max.map_or("NaN".to_string(), |v| format!("{:.3}", v)),
        fan_speed_avg.map_or("NaN".to_string(), |v| format!("{:.3}", v))
    ];
    if let Err(e) = wtr.write_record(data_row) {
        log_event(&format!("Failed to write record to CSV: {}", e), log_option);
        return false;
    }

    // Write the data
    if let Err(e) = wtr.flush() {
        log_event(&format!("Failed to flush CSV writer: {}", e), log_option);
        return false;
    }

    // If everything worked, return true.
    true
}