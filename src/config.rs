use std::{env, time::Duration};
use std::time::SystemTime;
use dotenvy::{dotenv, from_path};
use std::path::PathBuf;
use crate::functions::log_event;

pub fn load_var(dir: &Option<std::path::PathBuf>, fallback_dirs: &Vec<PathBuf>)

// Define output types.
-> (
    bool, // .................................... log_option
    u8, // ...................................... gpio_pin
    f32, f32, f32, // ........................... off_temp, min_temp, max_temp
    f32, f32, // ................................ min_duty_cycle, max_duty_cycle
    u64, // ..................................... period_micros
    u64, // ..................................... temp_cycle
    f32, f32, f32, f32, // ...................... factor_hot_old, factor_hot_new, factor_cold_old, factor_cold_new
    Duration, // ................................ delay_duration
    Duration, // ................................ error_duration
    bool, u64, u64, u64, f32, SystemTime, // .... stat_option, stat_cycle, stat_min_time, stat_max_time, stat_delta_t, stat_time_start
    u64, u64, f32, f32, // ...................... i, i_stat, n_on_fan, n_on_temp
    f32, f32, // ................................ duty_cycle, duty_cycle_old
    Option<f32>, Option<f32>, Option<f32>, // ... cpu_temp_min, cpu_temp_max, cpu_temp_avg
    f32, bool, // ............................... fan_state_stat, fan_state_prev
    Option<f32>, Option<f32>, Option<f32>  // ... fan_speed_avg, fan_speed_min, fan_speed_max
)

// Function.
{
    // implement this to try to get the .env working after moving to systemd folder?
    // https://docs.rs/dotenv/latest/dotenv/fn.from_path.html

    // Load .env file.
    
    let mut dotenv_success = false;

    if !dotenv_success {

        let dotenv_result = dotenv();
        match dotenv_result {
            Ok(_) => {
                println!(".env file loaded successfully using 'dotenv()'.");
                dotenv_success = true;
            }
            Err(e) => {
                println!("Failed to load .env file using 'dotenv()': {}", e);
            }
        }
    }

    if !dotenv_success {
        match dir {
            Some(path) => {
                let env_path: PathBuf = path.join(".env");
                if env_path.exists() {
                    let dotenv_result = from_path(&env_path);
                    match dotenv_result {
                        Ok(_) => {
                            println!(".env file loaded successfully from {}.", path.display());
                            dotenv_success = true;
                        }
                        Err(e) => {
                            println!("Failed to load .env file from {}: {}", path.display(), e);
                        }
                    }
                } else {}
            }
            None => {}
        }
    }

    if !dotenv_success {
        for path in fallback_dirs.iter() {
            // Construct the path to restart.sh.
            let env_path: PathBuf = path.join(".env");
            // Check if the restart script is located in the directory.
            if env_path.exists() {
                let dotenv_result = from_path(&env_path);
                match dotenv_result {
                    Ok(_) => {
                        println!(".env file loaded successfully from {}.", path.display());
                        dotenv_success = true;
                        break;
                    }
                    Err(e) => {
                        println!("Failed to load .env file from {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    if !dotenv_success {
        println!("Failed to load .env file. Fallback values for environmental variables will be used.");
    }


    // Define whether errors should be logged or not.
    let log_option: bool = env::var("log_option")
        .unwrap_or_else(|_| {"true".to_string()})
        .parse::<bool>()
        .ok()
        .filter(|&logoption| [true, false].contains(&logoption))
        .unwrap_or_else(|| {true});


    // Define the GPIO pin used for PWM control. The pin has to be either 12, 13, 18 or 19 with 18 as the default.
    let gpio_pin: u8 = env::var("gpio_pin")
        .unwrap_or_else(|_| {log_event("Warning: 'gpio_pin' not found in .env file. Using default: 18", log_option); "18".to_string()})
        .parse::<u8>()
        .ok()
        .filter(|&gpiopin| [12, 13, 18, 19].contains(&gpiopin))
        .unwrap_or_else(|| {log_event("Warning: Incorrect 'gpio_pin' defined, must be either 12, 13, 18 or 19. Using default: 18", log_option); 18});


    // Define the temperature where the fan turns off.
    // The default value is 45.0 degree C.
    let off_temp: f32 = env::var("off_temp")
        .unwrap_or_else(|_| {log_event("Warning: 'off_temp' not found in .env file. Using default: 45.0", log_option); "45.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'off_temp' defined, must be a number. Using default: 45.0", log_option); 45.0});


    // Define the temperature where the fan turns on.
    // The minimum temperature has to be at least equal to the off temperature with 50.0 degree C as the default value.
    let min_temp: f32 = env::var("min_temp")
        .unwrap_or_else(|_| {log_event("Warning: 'min_temp' not found in .env file. Using default: 50.0", log_option); "50.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'min_temp' defined, must be a number. Using default: 50.0", log_option); 50.0})
        .max(off_temp);


    // Define the temperature where the fan reaches full speed.
    // The maximum temperature has to be at least equal to the minimum temperature with 70.0 degree C as the default value.
        let max_temp: f32 = env::var("max_temp")
        .unwrap_or_else(|_| {log_event("Warning: 'max_temp' not found in .env file. Using default: 70.0", log_option); "70.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'max_temp' defined, must be a number. Using default: 70.0", log_option);70.0})
        .max(min_temp);


    // Define the minimum speed at which the fan will run.
    // The minimum fan speed has to be between 0.00 (0 percent) and 1.00 (100 percent) with 0.20 (20 percent) as the default value.
    let min_duty_cycle: f32 = env::var("min_duty_cycle")
        .unwrap_or_else(|_| {log_event("Warning: 'min_duty_cycle' not found in .env file. Using default: 0.20", log_option); "0.20".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'min_duty_cycle' defined, must be a number. Using default: 0.20", log_option); 0.20})
        .max(0.00 as f32)
        .min(1.00 as f32);


    // Define the maximum speed at which the fan will run.
    // The maximum fan speed has to be between the minimum fan speed and 1.00 (100 percent) with 1.00 (100 percent) as the default value.
    let max_duty_cycle: f32 = env::var("max_duty_cycle")
        .unwrap_or_else(|_| {log_event("Warning: 'max_duty_cycle' not found in .env file. Using default: 1.00", log_option); "1.00".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'max_duty_cycle' defined, must be a number. Using default: 1.00", log_option); 1.00})
        .max(min_duty_cycle)
        .min(1.00 as f32);


    // Define the frequency of the PWM signal.
    // The frequency has to be between 1 Hz and 1 MHz (due to further calculations and functions used) with 1 kHz as the default value.
    let pwm_freq: u64 = env::var("pwm_freq")
        .unwrap_or_else(|_| {log_event("Warning: 'pwm_freq' not found in .env file. Using default: 1000", log_option); "1000".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'pwm_freq' defined, must be an integer . Using default: 1000", log_option); 1000})
        .max(1 as u64)
        .min(1000000 as u64);


    // Calculate the duration of a single PWM period in microseconds.
    let period_micros = 1000000 / pwm_freq as u64;


    // Define how often the CPU temperature should be checked.
    // The frequency has to be between 1 Hz and 1 MHz (due to further calculations and functions used).
    // It also has to be smaller than and a divisor of the PWM frequency with 10 Hz as the default value.
    let temp_freq: u64 = env::var("temp_freq")
        .unwrap_or_else(|_| {log_event("Warning: 'temp_freq' not found in .env file. Using default: 10", log_option); "10".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'temp_freq' defined, must be an integer number. Using default: 10", log_option); 10})
        .max(0 as u64)
        .min(pwm_freq)
        .min(1000000 as u64);
    // If temp_freq is off (i.e. 0), the temperature should be read for every PWM cycle.
    let temp_freq = if temp_freq == 0 {
        pwm_freq
    // Ensure minimum frequency of 1 Hz.
    } else if temp_freq < 1 {
        1
    // Ensure that temp_freq is a divisor of pwm_freq, or adjust it to be the largest divisor.
    } else if pwm_freq % temp_freq != 0 {
        log_event("Warning: 'temp_freq' is not a divisor of 'pwm_freq'. Using largest valid divisor.", log_option);
        (1..=temp_freq).rev().find(|&i| pwm_freq % i == 0).unwrap_or(1)
    } else {temp_freq};


    // Calculate how many PWM cycles should pass before the temperature is measured each time.
    let temp_cycle: u64 = pwm_freq / temp_freq;


    // Define the dampening delay for rising temperatures.
    // The minimum delay is a single temperature reading period (in seconds). In this case, no delay will be applied. The default value is 1.0 s.
    let delay_hot: f32 = env::var("delay_hot")
        .unwrap_or_else(|_| {log_event("Warning: 'delay_hot' not found in .env file. Using default: 1.0", log_option); "1.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'delay_hot' defined, must be a number. Using default: 1.0", log_option); 1.0})
        .max(1.0 / temp_cycle as f32);


    // Define the dampening delay for falling temperatures.
    // The minimum delay is a single temperature reading period (in seconds). In this case, no delay will be applied. The default value is 10.0 s.
    let delay_cold: f32 = env::var("delay_cold")
        .unwrap_or_else(|_| {log_event("Warning: 'delay_cold' not found in .env file. Using default: 10.0", log_option); "10.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'delay_cold' defined, must be a number. Using default: 10.0", log_option); 10.0})
        .max(1.0 / temp_cycle as f32);


    // Calculate the factors to dampen fan speed changes.
    // If the delay is at or below a single PWM period, no dampening will be applied.
    let factor_hot_old = if delay_hot > 1.0 / (temp_freq  as f32) {
        1.0 - 1.0 / (temp_freq as f32 * delay_hot)
    } else {0.0 as f32};

    let factor_hot_new = if delay_hot > 1.0 / (temp_freq as f32) {
        1.0 / (temp_freq as f32 * delay_hot)
    } else {1.0 as f32};

    let factor_cold_old = if delay_cold > 1.0 / (temp_freq as f32) {
        1.0 - 1.0 / (temp_freq as f32 * delay_cold)
    } else {0.0 as f32};

    let factor_cold_new = if delay_cold > 1.0 / (temp_freq as f32) {
        1.0 / (temp_freq as f32 * delay_cold)
    } else {1.0 as f32};


    // Define the minimum duration between turning the fan on or off.
    // The minimum duration is 0 ms with 2000 ms as the default value.
    let delay_time: u64 =  env::var("delay_time")
        .unwrap_or_else(|_| {log_event("Warning: 'delay_time' not found in .env file. Using default: 2000", log_option); "2000".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'delay_time' defined, must be an integer. Using default: 2000", log_option); 2000})
        .max(0 as u64);


    // Convert delay_time to an actual duration.
    let delay_duration: Duration = Duration::from_millis(delay_time);


    // Define the minimum duration between error message outputs if no cpu temperature can be determined.
    // The minimum duration is 0 s with 60 s as the default value.
    let error_time: u64 = env::var("error_time")
        .unwrap_or_else(|_| {log_event("Warning: 'error_time' not found in .env file. Using default: 60", log_option); "60".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'error_time' defined, must be an integer. Using default: 60", log_option); 60})
        .max(0 as u64);


    // Convert error_time to an actual duration.
    let error_duration: Duration = Duration::from_secs(error_time);


    // Define whether statistics should be logged or not.
    let stat_option: bool = env::var("stat_option")
        .unwrap_or_else(|_| {log_event("Warning: 'stat_option' not found in .env file. Using default: true", log_option); "true".to_string()})
        .parse::<bool>()
        .ok()
        .filter(|&logoption| [true, false].contains(&logoption))
        .unwrap_or_else(|| {log_event("Warning: Incorrect 'stat_freq' defined, must be true or false. Using default: true", log_option); true});


    // Define how often statistics should be calculated.
    // The frequency has to be between 1 Hz and 1 MHz (due to further calculations and functions used).
    // It also has to be smaller than and a divisor of the PWM frequency with 10 Hz as the default value.
    let stat_freq: u64 = env::var("stat_freq")
        .unwrap_or_else(|_| {log_event("Warning: 'stat_freq' not found in .env file. Using default: 10", log_option); "10".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'stat_freq' defined, must be an integer number. Using default: 10", log_option); 10})
        .max(0 as u64)
        .min(pwm_freq)
        .min(1000000 as u64);
    // If stat_freq is off (i.e. 0), the statistics should be calculated for every PWM cycle.
    let stat_freq = if stat_freq == 0 {
        pwm_freq
    // Ensure minimum frequency of 1 Hz.
    } else if stat_freq < 1 {
        1
    // Ensure that stat_freq is a divisor of pwm_freq, or adjust it to be the largest divisor.
    } else if pwm_freq % stat_freq != 0 {
        log_event("Warning: 'stat_freq' is not a divisor of 'pwm_freq'. Using largest valid divisor.", log_option);
        (1..=stat_freq).rev().find(|&i| pwm_freq % i == 0).unwrap_or(1)
    } else {stat_freq};
    // Calculate how many PWM cycles should pass before statistics are calculated each time.
    let stat_cycle: u64 = pwm_freq / stat_freq;


    // Define the minimum duration after which statistics are saved.
    // The value must be a whole number (integer) representing a duration in seconds, ranging from 1 second to ~ 1 year with 60s as the default value. 
    let stat_min_time: u64 = env::var("stat_min_time")
        .unwrap_or_else(|_| {log_event("Warning: 'stat_min_time' not found in .env file. Using default: 60", log_option); "60".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'stat_min_time' defined, must be an integer . Using default: 60", log_option); 60})
        .max(1 as u64)
        .min(31536000 as u64);


    // Define the maximum duration after which statistics are saved.
    // The value must be a whole number (integer) representing a duration in seconds, ranging from 1 second to ~ 1 year and at least as long as the minimum duration.
    // The default value is 3600s. 
    let stat_max_time: u64 = env::var("stat_max_time")
        .unwrap_or_else(|_| {log_event("Warning: 'stat_max_time' not found in .env file. Using default: 3600", log_option); "3600".to_string()})
        .parse::<u64>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'stat_max_time' defined, must be an integer . Using default: 3600", log_option); 3600})
        .max(1 as u64)
        .min(31536000 as u64)
        .max(stat_min_time);


    // Define the minimum difference in min and max temperatures for statistics to be saved before the max duration.
    let stat_delta_t: f32 = env::var("stat_delta_t")
        .unwrap_or_else(|_| {log_event("Warning: 'stat_delta_t' not found in .env file. Using default: 5.0", log_option); "5.0".to_string()})
        .parse::<f32>()
        .unwrap_or_else(|_| {log_event("Warning: Incorrect 'stat_delta_t' defined, must be a number. Using default: 5.0", log_option); 5.0})
        .max(0.0);


    // Define different integers for iterations.
    let i: u64 = 1; // ............. Counts the total loops of the main function (i.e. +=1 with every PWM cycle).
                                  // Together with stat_cycle and temp_cycle used to determine when temperatures should be read and when statistics should be calculated.
    let i_stat: u64 = 1; // ........ Used to count how often statistics where calculated. Used to calculate the on time of the fan.
    let n_on_fan: f32 = 0.0; // .... Used to count how often the fan was running when calculating statistics.
                                  // Average fan speed is only calculated for the number of times the fan was running.
    let n_on_temp: f32 = 0.0; // ... Used to count the number of valid temperature readings for statistics. Average is only calculated for the number of valid readings.


    // Define the variable for the fan speed and set it to the max cycle to initially keep the fan to full power.
    let duty_cycle: f32 = max_duty_cycle;
    let duty_cycle_old: f32 = max_duty_cycle;


    // Define the statistics variables.
    let cpu_temp_min: Option<f32> = None;
    let cpu_temp_max: Option<f32> = None;
    let cpu_temp_avg: Option<f32> = None;
    let fan_state_stat: f32 = 1.0;
    let fan_state_prev: bool = false;
    let fan_speed_avg: Option<f32> = None;
    let fan_speed_min: Option<f32> = None;
    let fan_speed_max: Option<f32> = None;
    let stat_time_start = SystemTime::now();
    

    // Output arguments
    (
        log_option,
        gpio_pin,
        off_temp, min_temp, max_temp,
        min_duty_cycle, max_duty_cycle,
        period_micros,
        temp_cycle,
        factor_hot_old, factor_hot_new, factor_cold_old, factor_cold_new,
        delay_duration,
        error_duration,
        stat_option, stat_cycle, stat_min_time, stat_max_time, stat_delta_t, stat_time_start,
        i, i_stat, n_on_fan, n_on_temp,
        duty_cycle, duty_cycle_old,
        cpu_temp_min, cpu_temp_max, cpu_temp_avg,
        fan_state_stat, fan_state_prev,
        fan_speed_avg, fan_speed_min, fan_speed_max
    ) 
}

/*

*/