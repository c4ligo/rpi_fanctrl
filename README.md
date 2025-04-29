
# rpi_fanctrl: Raspberry Pi PWM Fan Control

rpi_fanctrl allows for temperature based fan control using the GPIO pins of a Raspberry Pi.

I wrote this program after I accidentally tore off the fan connector from my Raspberry Pi 5 and still wanted to use the official Raspberry Pi Active Cooler with PWM controlled fan speed based on cpu temperature.

---

## Table of Contents

[Functions](#functions) | [Hardware](#supportedtested-hardware) | [Wiring](#wiring) | [Install](#install) | [Environmental Variables](#environmental-variables) | [Wrapper](#fanctrl-wrapper) | [Systemd Service](#default-systemd-service-settings) | [Contributions](#contributions) | [History](#release-history)

---

## Functions

[🔝 Back to Table of Contents](#table-of-contents)

- **PWM fan control:** Starts, stops, and controls the speed of a fan based on CPU temperature.
- **Smooth operation:** Fan speed is linearly increased with temperature. Responses are dampened to prevent sudden changes in fan speed and can be adjusted to react faster or slower to either rising or falling temperatures. Rapid on and off switching of the fan is also prevented.
- **Logging options:** Optionally, the program can log statistics such as cpu temperature and fan speed as well as any errors if there should be any.
- **CPU efficient:** The program is written in Rust and has approx. 0.4% CPU utilization.
- **Easily configurable:** Most everything can be configured using environment variables.
- **systemd service:** Automatic install sets everything up as a persistent systemd service.

---

## Supported/Tested Hardware

[🔝 Back to Table of Contents](#table-of-contents)

- Raspberry Pi 5 (Raspberry OS)
- Raspberry Pi Active Cooler

Currently, this is the only hardware I have tested this on.
However, this program uses the [rppal crate (0.22.1)](https://docs.rs/rppal/0.22.1/rppal/) to control fan speed which does support most Raspberry Pi models.
If I have the chance to test it with other Raspberries and fans, or if anyone else can provide feedback, I will update this list.

---

## Wiring

[🔝 Back to Table of Contents](#table-of-contents)

Default configuration for the official [Raspberry Pi Active Cooler](https://www.raspberrypi.com/products/active-cooler/). Other fans should also work with this wiring.

- "Pin #" are the board pin numbers.
- "GPIO #" are the GPIO pin numbers.
- "PWM Signal" must be on a PWM capable GPIO pin:
  - GPIO 18 (Pin 12) - Default
  - GPIO 19 (Pin 35)
  - GPIO 12 (Pin 32)
  - GPIO 13 (Pin 33)
- More information on the Raspberry Pi pins can be found [here](https://pinout.xyz/pinout/pin35_gpio19/) and [here](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#gpio).

**How to connect the fan to the pins of the Raspberry Pi:**
Depending on the fan, some soldering may be required. For the official fan, the wires have to be cut from the fan connector and soldered to female jumper cables. The stated colors of the cables are for the official fan and might be different for other fans.

- Connect the power wire (red) of the fan to any of the 5V pins (e.g. pin 4).
- Connect the ground wire (black) of the fan to any of the ground pins (e.g. pin 6).
- Connect the PWM signal wire (blue) of the fan to any PWM capable GPIO pins. The program expects GPIO 18 by default, so if you use another pin, this has to be changed in the environmental variables.

![Schematics wiring diagram of a fan connected to the GPIO pins of a raspberry pi](/assets/rpi_fanctrl_wiring.png)

The official fan also has a yellow wire. This is used for the tachometer signal (?) which is not currently used for this program. Therefore, the yellow wire is not connected to any pin.

---

## Install

[🔝 Back to Table of Contents](#table-of-contents)

An automatic install script `install.sh` is included which compiles and installs the `rpi_fanctrl` program as a systemd service and set everything up including default environmental variables as well as wrapper script `fanctrl` for convenient management.

- Ensure that `install.sh` is executable:

    In the terminal go to the downloaded rpi_fanctrl directory containing the install script (assuming the directory is located in your home directory):

    ```Bash
    ~ $ cd rpi_fanctrl
    ```

    Then run the following to make the script executable:

    ```Bash
    ~/rpi_fanctrl $ chmod +x install
    ```

    Using the GUI, right click the script file and select 'Properties'. Then go to the 'Permissions' tab and set 'Execute' to 'Anyone'.

    ![Making the install.sh file executable using the GUI in the Raspberry Pi OS - Part 1: Right click the file and select 'Properties'.](/assets/rpi_fanctrl_guiinstallexec_1.png) | ![Making the install.sh file executable using the GUI in the Raspberry Pi OS - Part 2: Set 'Execute' to 'Anyone' in the 'Permissions' tab.](/assets/rpi_fanctrl_guiinstallexec_2.png)
    -|-

- Start the install script:

    If you used the terminal to make the script executable, next run:

    ```Bash
    ~/rpi_fanctrl $ ./install.sh
    ```

    Using the GUI, double click the script file and select 'Execute in Terminal'.

    ![Running the install.sh script using the GUI in the Raspberry Pi OS by double clicking the file and selecting 'Execute in Terminal'.](/assets/rpi_fanctrl_guiinstall.png)

The install script will then guide you through the installation.

---

## Environmental Variables

[🔝 Back to Table of Contents](#table-of-contents)

The following environmental variables can be defined:

Variable | Function
-|-
**gpio_pin** | Defines which GPIO pin is used for the PWM signal. Can be either 18, 19, 12 or 13.
**off_temp** | The CPU temperature in °C below which the fan is stopped.
**min_temp** | The CPU temperature in °C above which the fan is started (if it is not already running). Has to be equal to, or larger than off_temp.
**max_temp** | The CPU temperature in °C at which the fan is spinning with the maximum defined speed. Has to be equal to, or larger than min_temp.
**min_duty_cycle** | The minimum fan speed (i.e. the fan speed at the min_temp). Has to be between 0.0 (fan off) and 1.0 (full speed).
**max_duty_cycle** | The maximum fan speed (i.e. the fan speed at the max_temp). Has to be between 0.0 (fan off) and 1.0 (full speed). Has to be equal to, or larger than min_duty_cycle.
**pwm_freq** | The frequency of the PWM signal (depends on the fan used). Has to be between 1 (i.e 1 Hz) and 1000000 (i.e. 1 MHz). For the Raspberry Pi Active Cooler I found a value of 1000 to work very well.
**temp_freq** | The frequency at which the CPU temperature should be checked. Has to be between 1 (i.e 1 Hz) and 1000000 (i.e. 1 MHz) and furthermore it has to be smaller than, and a divisor of, pwm_freq.
**delay_hot** | The dampening in seconds for the fan speed response to rising CPU temperatures. The minimum value is 1 / temp_freq (i.e. no dampening). *Example*: The fan speed for 50°C is 25% and 75% for 60°C. In case of a sudden temperature increase, the fan speed would immediatly increase from 25% to 75%. With a delay of 1 s, it takes 1 s for the fan speed to adjust to the temperature. With a delay of 10 s, it will take 10 s, and so on ... ![A graph showing the differences in fan speed response to temperature changes for different delay values.](/assets/rpi_fanctrl_temp_delay.png) This delay value serves to smoothen the fan speed response and to prevent rapid fan speed changes.
**delay_cold** | The dampening in seconds for the fan speed response to falling CPU temperatures. The minimum value is 1 / temp_freq (i.e. no dampening). It is advised to have a larger delay for falling temperature values than for rising values. In this case the fan will quickly increase in speed in case of rising temperatures and keep spinning faster longer, even when temperatures drop again.
**delay_time** | The minimum duration in ms between turing the fan on and of. Must be equal to or larger than 0. This prevents the fan from turning rapidly on and off.
**error_time** | The minimum duration in s between error message outputs if no cpu temperature can be determined. Must be equal to or larger than 0.
**log_option** | Whether errors or other program messages should be logged or not. Must be either 'true' or 'false'.
**stat_option** | Whether statistics should be logged or not. Must be either 'true' or 'false'.
**stat_freq** | The frequency which stats are checked. Has to be between 1 (i.e 1 Hz) and 1000000 (i.e. 1 MHz) and furthermore it has to be smaller than, and a divisor of, pwm_freq.
**stat_min_time** | The minimum duration in s that has to pass before stats are logged. Has to be between 1 s and 3153600 s (~ 1 year).
**stat_max_time** | The maximum duration in s that can pass before stats are logged. Has to be between 1 s and 3153600 s (~ 1 year) and larger than stat_min_time.
**stat_delta_t** | The difference in min. and max. CPU temperature in a stat loggin period which determines when stats are logged. When the actual difference in larger than this value and the loggin period is already longer than the minimum duration, stats are logged and a new loggin period starts.

In case variables are missing or incorrect values are defined, sensibel defaults will be applied automatically by the program. In case logging is enabled, this will be logged.

---

## `fanctrl` wrapper

[🔝 Back to Table of Contents](#table-of-contents)

The `fanctrl` wrapper is automatically installed by the install script and added to the PATH.
Therefore, the wrapper can simply be accessed in the terminal by running either **`fanctrl`** or **`fanctrl --option`**.

The following options are available:

Option | Function
-|-
**--start** | Start rpi_fanctrl if not already running
**--stop** | Stop rpi_fanctrl if it is running
**--restart** | Restart rpi_fanctrl if it is running, else start it
**--env** | Edit the environment file and optionally restart rpi_fanctrl
**--systemd** | Edit the systemd service file and optionally reload systemd and restart the service
**--stat** | View the statistics file if logging statistics is enabled
**--log** | View the log file if logging is enabled
**--info** | View information on the directories where the program files are installed
**--uninstall** | Uninstall the rpi_fanctrl program

---

## Default systemd service settings

[🔝 Back to Table of Contents](#table-of-contents)

The following systemd service settings are applied during default installation:

Systemd Service Setting | Function
-|-
After=multi-user.target | Start the service after the multi-user.target is reached (when most basic services are up and running).
Restart=on-failure | If the service crashes or fails (i.e. exits with a non-zero status), systemd will automatically restart it. It will not restart if the service stops cleanly.
RestartSec=10 |  After a failure, systemd will wait 10 seconds before trying to restart the service.

---

## Contributions

[🔝 Back to Table of Contents](#table-of-contents)

Contributions are very welcome.

Especially welcome would be:

- Test the program on different hardware (i.e. other Raspberry Pi models and other fans) and report if it is working or find out/help find out/fix why it is not working. For fans, what PWM frequency works best.
- The program currently does not handle SIGHUP signals very elegantly. The SIGHUP signal should trigger a restart of the program. The current implementation is to end the program with `process::exit(1)` in order to trigger the restart by systemd (if using the default service settings). If anyone could help with this, that would be great!

---

## Release History

[🔝 Back to Table of Contents](#table-of-contents)

Version | Changes
-|-
1.0.0 | First release
