// Public crates.
use rppal::gpio::Gpio;
use std::{sync::{Arc, Mutex, mpsc}, thread, time};
use std::time::SystemTime;
use std::process;
// use std::process::Command;
use std::env;
// use ctrlc;
use signal_hook::consts::signal::{SIGINT, SIGTERM, SIGHUP};
use signal_hook::iterator::Signals;
use std::path::PathBuf;

// Own crates.
mod functions;
mod config;
use functions::*;
use config::load_var;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Get the directory of the executed binary.
    // This path is required for loading the environmental variables, saving logs and stats.
    let dir = std::env::current_exe()
    .ok()
    .and_then(|p| p.parent().map(|d| d.to_path_buf()));


    // Get the username and use it to construct the home directory.
    // This is used as a fallback in case the binary directory could not be determined or if the other required files (logs, stats, scripts) are not in the same directory as the binary.
    // If the username cannot be determined the standard "pi" user is assumed.
    let home_dir = format!("/home/{}", env::var("USER").unwrap_or("pi".to_string()));
    // Define multiple fallback directories
    let fallback_dirs = vec![
        PathBuf::from("/usr/local/bin/rpi_fanctrl/"),
        PathBuf::from(home_dir)
    ];


    // Load variables.
    let(
        log_option,
        gpio_pin,
        off_temp, min_temp, max_temp,
        min_duty_cycle, max_duty_cycle,
        period_micros,
        temp_cycle,
        factor_hot_old, factor_hot_new, factor_cold_old, factor_cold_new,
        delay_duration,
        error_duration,
        stat_option, stat_cycle, stat_min_time, stat_max_time, stat_delta_t, mut stat_time_start,
        mut i, mut i_stat, mut n_on_fan, mut n_on_temp,
        mut duty_cycle, mut duty_cycle_old,
        mut cpu_temp_min, mut cpu_temp_max, mut cpu_temp_avg,
        mut fan_state_stat, mut fan_state_prev,
        mut fan_speed_avg, mut fan_speed_min, mut fan_speed_max
    ) = load_var(&dir, &fallback_dirs);

    log_event("Fan control initialized.", log_option);

    
    // If the directory of the binary could not be determined, log it.
    if dir.is_none() {log_event("Could not get the directory of the binary.", log_option);};


    // In case a SIGHUP signal is received use this variable to "force" systemd to restart by exiting with a non 0 exit code. 
    let mut restart = false;


    // Create the stats file if statistics are logged.
    let file_path = "stats.csv";
    if stat_option {
        // Create the .csv file and add the header (will only be performed if the file does not already exist).
        create_csv(file_path, log_option);
    }


    // Set GPIO pin.
    let gpio = Gpio::new().unwrap_or_else(|_| {log_event("Warning: Failed to initialize GPIO. Exiting.", log_option); std::process::exit(1);});
    let pin = gpio.get(gpio_pin).unwrap_or_else(|_| {log_event(&format!("Warning: Failed to access GPIO pin {}. Exiting.", gpio_pin), log_option);std::process::exit(1);}).into_output();
    let pin = Arc::new(Mutex::new(pin));
    

    // Immediately start the fan on full power and set the initial fan state.
    if let Ok(mut pin) = pin.lock() {
        pin.set_high();
    }
    let mut fan_on = true;


    // Set the initial time point of the last fan state.
    // The delay duration is subtracted so that the fan can start immediately.
    let mut state_change_time = SystemTime::now()
        .checked_sub(delay_duration)
        .unwrap_or_else(|| {log_event("Error: Failed to subtract delay from current time. Using current time as fallback.", log_option);SystemTime::now()});


    // Set the initial time point of the last cpu temperature error.
    // The delay duration is subtracted so that the first error message could be sent immediately.
    let mut last_error_time = SystemTime::now().checked_sub(error_duration).unwrap_or_else(SystemTime::now);


    // Define the temperature variable and get the initial CPU temperature.
    let mut cpu_temp_missing = false;
    let mut cpu_temp = read_cpu_temperature();
    cpu_temp = match cpu_temp {
        Some(temp) => Some(temp),
        None => {
            if let Ok(mut pin) = pin.lock() {
                pin.set_high();
            }
            if last_error_time.elapsed().unwrap_or_default() >= error_duration {
                log_event("Failed to read CPU temperature. Turning fan on at full power.", log_option);
                last_error_time = SystemTime::now();
            }
            None
        }
    };


    // Define variables for comparison of current temperature with previous temperature.
    let mut cpu_temp_old = cpu_temp;
    let mut same_temps = true;
        _ = same_temps;   
    if cpu_temp == None{
        cpu_temp_missing = true
    }


    // Setup message channels. 
    let (tx, rx) = mpsc::channel();   // For the shutdown flag.
    let (txr, rxr) = mpsc::channel(); // For the reload flag.


    // Setup handler for the SIGINT/SIGTERM/SIGHUP signals.
    let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGHUP])?;
    let handle = signals.handle();
    

    // Move the handler to a separate thread and listen for signals.
    // If either SIGINT or SIGTERM are received, send the shutdown flag. If SIGHUP is received, send the reload flag.
    thread::spawn(move || {
        for signal in signals.forever() {
            #[allow(non_snake_case)] // Unix signals are all upper case, and are mapped correspondingly in the signal_hook crate.
            match signal {
                SIGINT | SIGTERM => {
                    let shdn_flag: bool = true;
                    tx.send(shdn_flag).unwrap();
                },
                SIGHUP => {
                    let rld_flag: bool = true;
                    txr.send(rld_flag).unwrap();
                },
                _ => unreachable!(),
            }
        }
    });


    // Check CPU temperature and adjust fan speed accordingly.
    // This loop runs the whole time while the program is running.
    // Each loop cycle should be one PWM period + calculation time long.
    loop {


        // Check if the shutdown flag was sent.
        // If yes, save stats (if logged), break the loop and exit the program.
        let shdn_recv: bool = rx.try_recv().unwrap_or(false);
        

        // Check if the reload flag was sent.
        // If yes, save stats (if logged) and reload environmental variables.
        let rld_recv: bool = rxr.try_recv().unwrap_or(false);


        // Read temperature for the first cycle and then every n-th cycle.
        // This is implemented to optimize performance.
        if i % temp_cycle == 0 || i == 1 {
            cpu_temp = read_cpu_temperature();
            cpu_temp = match cpu_temp {
                Some(temp) => Some(temp),
                // If there was no valid temperature reading, set the fan to high.
                None => {
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_high();
                    }
                    if last_error_time.elapsed().unwrap_or_default() >= error_duration {
                        log_event("Failed to read CPU temperature. Turning fan on at full power.", log_option);
                        last_error_time = SystemTime::now();
                    }
                    None
                }
            };
            // Check if temperature could be read.
            if cpu_temp == None{
                cpu_temp_missing = true
            } else {
                cpu_temp_missing = false
            }
        // For every cycle where the temperature is not read, use the previous value.
        } else {
            cpu_temp = cpu_temp_old;
        }


        // Check if the current and the previous temperature are the same.
        // Only returns true if both are some value. 
        same_temps = cpu_temp_old.zip(cpu_temp).map_or(false, |(old, new)| old == new);


        // The cpu temperature before dampening is used for statistics.
        let cpu_temp_orig = cpu_temp;

        
        // Only adjust fan speed if there is a valid cpu temperature reading.
        // Else, the fan was already set to full speed.
        // Temperature check will be repeated in one PWM period.
        if let Some(mut cpu_temp) = cpu_temp {
            

            // Dampen fan speed changes.
            // Fast fan speed changes or frequent acceleration and deceleration of the fan should be avoided.
            // Since fan speed directly correlates to CPU temperature, the change rate of the temperature is dampened.
            // Only do this if there is a valid value for cpu_temp_old.
            if let Some(cpu_temp_old) = cpu_temp_old {
                cpu_temp = if same_temps {
                    // If both temperatures are the same, use the current one without dampening.
                    cpu_temp
                } else if cpu_temp < cpu_temp_old {
                    // If the current temperature is colder than the previous one, dampen using the cold factor.
                    cpu_temp * factor_cold_new + cpu_temp_old * factor_cold_old
                } else {
                    // If the current temperature is hotter than the previous one, dampen using the hot factor.
                    cpu_temp * factor_hot_new + cpu_temp_old * factor_hot_old
                };
            };

            // Assign the dampened cpu temperature as the previous one for the next cycle,
            cpu_temp_old = Some(cpu_temp);


            // Calculate duty cycle.
            // The speed at which the fan should be turning is calculated based on the dampened CPU temperature.
            duty_cycle = if same_temps {
                // If the temperatures did not change, do not recalculate the fan speed but use the previous one.
                duty_cycle_old
            } else if cpu_temp <= min_temp {
                // Below the min temperature, use the min speed.
                min_duty_cycle
            } else if cpu_temp >= max_temp {
                // Above the max temperature, use the max speed.
                max_duty_cycle
            } else if cpu_temp > min_temp && cpu_temp < max_temp && min_duty_cycle < max_duty_cycle {
                // Between min and max temperature, use linear interpolation to calculate the fan speed.
                min_duty_cycle + (cpu_temp - min_temp) * (max_duty_cycle - min_duty_cycle) / (max_temp - min_temp)
            } else {
                // As a fallback, go full speed. This Option should not be required.
                max_duty_cycle
            };

            // Assign the current fan speed as the previous on for the next cycle.
            duty_cycle_old = duty_cycle;


            // Calculate active and inactive times.
            // Using PWM, the fan is either turned on or turned off.
            // To regulate fan speed, the fan is turned on and off rapidly.
            // E.g.: For a PWM frequency of 1 kHz, the fan is turned on and off 1000 times per second.
            // Tu run the fan at 50 percent, the on time and the off time is the same.
            // E.g.: For a PWM frequency of 1 kHz, this means that fan is on for 500 micro s and off for 500 micro s.
            let active_time = (duty_cycle * period_micros as f32) as u64;
            let inactive_time = period_micros - active_time;


            // Time since the last state change of the fan.
            // To avoid frequently turning the fan on and off, the duration since the last state change is calculated.
            // Not to be confused with on off in the context of regulating fan speed using PWM.
            let elapsed_time = state_change_time.elapsed().unwrap_or_default();


            // Adjust the fan speed.
            // According to the dampened CPU temperature, the current fan state and the duration since the last state change.
            // The loop is not repeated. It is always stopped with a break signal in its initial iteration.
            loop {


                // Keep the fan off.
                // If the fan is not running and the temperature is below the minimum temperature, keep the fan off.
                if !fan_on && cpu_temp <= min_temp {
                    thread::sleep(time::Duration::from_micros(period_micros));
                    break;
                }


                // Keep the fan on.
                // If the fan is running and the temperature is above the off temperature, keep the fan running.
                // Fan speed is set to the calculated speed based on the dampened temperature.
                if fan_on && cpu_temp > off_temp {
                    fan_on = true;
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_low();
                    }
                    thread::sleep(time::Duration::from_micros(inactive_time));
                    if let Ok(mut pin) = pin.lock() {
                                pin.set_high();
                            }
                    thread::sleep(time::Duration::from_micros(active_time));
                    break;
                }


                // Turn the fan off.
                // The fan is running but the temperature is below the off temperature.
                // If the time since the fan was started is above the delay duration, turn the fan off.
                if fan_on && cpu_temp <= off_temp && elapsed_time >= delay_duration {
                    state_change_time = SystemTime::now();
                    fan_on = false;
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_low();
                    }                    
                    thread::sleep(time::Duration::from_micros(period_micros));
                    break;
                }


                // Turn the fan on.
                // The fan is not running but the temperature is above the minimum temperature.
                // If the time since the fan was stopped is above the delay duration, turn the fan on.
                if !fan_on && cpu_temp > min_temp && elapsed_time >= delay_duration  {
                    state_change_time = SystemTime::now();
                    fan_on = true;
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_low();
                    }
                    thread::sleep(time::Duration::from_micros(inactive_time));
                    if let Ok(mut pin) = pin.lock() {
                                pin.set_high();
                            }
                    thread::sleep(time::Duration::from_micros(active_time));
                    break;
                }


                // Keep the fan on, due to time delay.
                // The fan is running and the temperature is below the off temperature.
                // However, the time since the fan was started is still below the delay duration.
                // The fan will keep running and the fan speed is set to the calculated speed based on the dampened temperature.
                if fan_on && cpu_temp <= off_temp && elapsed_time < delay_duration {
                    fan_on = true;
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_low();
                    }
                    thread::sleep(time::Duration::from_micros(inactive_time));
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_high();
                    }
                    thread::sleep(time::Duration::from_micros(active_time)); 
                    break;
                }


                // Keep the fan off, due to time delay.
                // The fan is not running and the temperature is above the minimum temperature.
                // However, the time since the fan was stopped is still below the delay duration.
                // The fan will not start running but stay off.
                if !fan_on && cpu_temp > min_temp && elapsed_time < delay_duration  {
                    /* fan_on = false;
                    if let Ok(mut pin) = pin.lock() {
                        pin.set_low();
                    } */
                    thread::sleep(time::Duration::from_micros(period_micros));
                    break;
                }


                // In case none of the other options triggered, wait and break.
                // This should not be required since all possibilities should be covered by the previous options.
                thread::sleep(time::Duration::from_micros(period_micros));
                break;
            }


        // If there was no valid temperature reading, wait for one period.    
        } else {thread::sleep(time::Duration::from_micros(period_micros));}


        // Log statistics if the option is set to true.
        // Statistics are logged on the first iteration of each stats period and there each nth iteration.
        // If a SIGINT/SIGTERM/SIGHUP signal is received, stats are always logged and written to teh .csv file.
        if stat_option && (i % stat_cycle == 0 || i_stat == 1 || (shdn_recv | rld_recv)) {
            i_stat += 1;

            log_statistics(
                
                // Input arguments
                &mut i_stat, &mut n_on_temp, &mut n_on_fan,
                stat_min_time, stat_max_time, stat_delta_t, &mut stat_time_start,
                log_option,
                duty_cycle,
                cpu_temp_missing,
                cpu_temp_orig, &mut cpu_temp_min, &mut cpu_temp_max, &mut cpu_temp_avg,
                fan_on,
                &mut fan_state_stat, &mut fan_state_prev,
                &mut fan_speed_min, &mut fan_speed_max, &mut fan_speed_avg,
                file_path, shdn_recv, rld_recv
            );
        }
        i += 1;


        // If a SIGINT/SIGTERM/SIGHUP signal was received, leave the main loop.
        // For a SIGINT/SIGTERM signal, nothing more is required, the program will shut down.
        if shdn_recv {
            log_event("SIGINT/SIGTERM signal received. Turning fan off and exiting program.", log_option);
            break;
        }

        if rld_recv {
            log_event("SIGHUP signal received. Restarting program to reload environmental variables. ", log_option);
            restart = true;
            break;
        }
    }

    
    // When the program leaves the main loop, it will shut down.
    // Ensure that the fan is off.
    if let Ok(mut pin) = pin.lock() {
        pin.set_low();
    }
    thread::sleep(time::Duration::from_micros(period_micros));

    
    // Clean up the signal_hook handle.
    handle.close();


    // In case a SIGHUP signal is received the program is exited with a non 0 exit code to "force" the restart by systemd.
    // This is a very bad implementation but I could not get it to work any other way :(
    if restart {
        log_event("Exiting program with exit(1). Program should be restarted by systemd.", log_option);
        process::exit(1);
    }
    
    // Program ends ... as all things must.
    log_event("Exiting program. Thank you and goodbye! Hope to see you again soon!", log_option);
    Ok(())
}