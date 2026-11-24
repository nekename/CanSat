# CanSat ESP32-S3 Rust Flight Computer

This repository contains the autonomous telemetry collection & transmission and airbag deployment system developed by Team Probodobodyne for the 2026 UK CanSat competition.

Rust is used in this project in order to effectively utilise the dual-core processor of the ESP32-S3 for uncompromised IMU integration without the mental overhead of programming multi-core operation in C++.

A Kalman filter estimates altitude accurately by fusing barometric pressure, vertical acceleration from the IMU, and GPS altitude.

### Electronic components and datasheet references

- Espressif ESP32-S3: [AliExpress](https://www.aliexpress.com/item/1005010800775978.html)
	- [Espressif ESP32-S3 Series Datasheet](https://documentation.espressif.com/esp32-s3_datasheet_en.pdf)
- Bosch BMP581: [Pimoroni](https://shop.pimoroni.com/products/adafruit-bmp581-i2c-or-spi-temperature-and-pressure-sensor-stemma-qt?variant=55993179767163)
	- [Bosch Sensortec BST-BMP581-DS001](https://www.bosch-sensortec.com/products/environmental-sensors/pressure-sensors/bmp581/)
- Adafruit LSM6DSOX + LIS3MDL 9 DoF IMU: [Pimoroni](https://shop.pimoroni.com/products/adafruit-lsm6dsox-lis3mdl-precision-9-dof-imu-stemma-qt-qwiic?variant=31594896851027)
	- [STMicroelectronics LSM6DSOX Datasheet](https://www.st.com/resource/en/datasheet/lsm6dsox.pdf)
	- [STMicroelectronics LIS3MDL Datasheet](https://www.st.com/resource/en/datasheet/lis3mdl.pdf)
- Pimoroni PA1010D GPS: [Pimoroni](https://shop.pimoroni.com/products/pa1010d-gps-breakout?variant=32257258881107)
	- [CD-PA1010D Datasheet](https://cdn-learn.adafruit.com/assets/assets/000/084/295/original/CD_PA1010D_Datasheet_v.03.pdf?1573833002)
- Waveshare Core1262: [Waveshare](https://www.waveshare.com/core1262-868m.htm)
	- [Semtech SX1261/2 Long Range Low Power Transceiver](https://www.semtech.com/products/wireless-rf/lora-connect/sx1262)
	- [Waveshare Core1262-868M Wiki & Schematics](https://www.waveshare.com/wiki/Core1262-868M)
- Adafruit SPI Flash SD Card: [Pimoroni](https://shop.pimoroni.com/products/adafruit-spi-flash-sd-card?variant=53511868121467)
	- [Adafruit Learning System Guide](https://learn.adafruit.com/adafruit-spi-flash-sd-card)

## Architecture

### Core 0:

- Reads barometric pressure and temperature from the BMP581 at 50 Hz.
- Runs the altitude-estimation Kalman filter using barometric, accelerometric, and GPS measurements.
- Deploys the airbag once the CanSat returns to the required altitude.
- Encodes and transmits telemetry packets once per second via the Core1262 LoRa module at 868.0 MHz.
- Logs CSV flight frames to an SD-compatible FAT filesystem in case of LoRa signal loss.

### Core 1:

- Dedicated thread pinned to Core 1
- Acquires 9-DOF motion data: linear acceleration and angular rates from the LSM6DSOX, plus magnetic field from the LIS3MDL.
- Updates the shared flight state thread-safely using Rust's `std::sync`.

## Pin mappings

GPIO 26 through 32 are reserved internally on the ESP32-S3 for flash/PSRAM and GPIO 0, 3, 45, and 46 are strapping pins.

| Peripheral       | Signal   | ESP32-S3 SuperMini Pin |
| :--------------- | :------- | :--------------------- |
| Shared I2C Bus   | SDA      | GPIO 8                 |
|                  | SCL      | GPIO 9                 |
| Shared SPI Bus   | SCK      | GPIO 12                |
|                  | MOSI     | GPIO 11                |
|                  | MISO     | GPIO 13                |
| Core1262 LoRa    | NSS (CS) | GPIO 10                |
|                  | NRESET   | GPIO 5                 |
|                  | BUSY     | GPIO 4                 |
|                  | DIO1     | GPIO 1                 |
| SD card          | CS       | GPIO 7                 |
| Airbag ESC       | PWM      | GPIO 6                 |
| Status Indicator | LED      | GPIO 48                |

## Peripheral details

### Bosch BMP581 (barometer & thermometer)

- [bmp581 Rust crate](https://crates.io/crates/bmp581)
- Oversampling:
	- Pressure: 16x (increases precision for altitude calculations)
	- Temperature: 2x (sufficient for small changes without wasting power)
- Output data rate: 25 Hz (40ms period)

### Adafruit LSM6DSOX + LIS3MDL (9-DoF IMU)

- LSM6DSOX accelerometer and gyroscope:
	- [lsm6dsox Rust crate](https://crates.io/crates/lsm6dsox)
		- Crate uses incorrect scaling factor for ±16 g so this is manually adjusted
	- Full scale:
		- Accelerometer: ±16 g (0.488 milli-g/LSB) as CanSat experiences acceleration of up to 20 g
		- Gyroscope: ±2000 dps (70 mdps/LSB) as CanSat can tumble rapidly
	- Output data rate: 104 Hz (rapid polling for accurately timed event triggers)
- LIS3MDL magnetometer:
	- [lis3mdl Rust crate](https://crates.io/crates/lis3mdl)
		- Crate uses incorrect I2C address so this is manually corrected
	- Full scale: ±16 gauss (permanent magnets and BLDC motor coils create high noise)
	- Output data rate: 40 Hz (25ms period)

### Pimoroni PA1010D (GPS)

- [nmea Rust crate](https://crates.io/crates/nmea)
- I2C address: `0x10`
- Requires conversion of speed data from knots to m/s

### Semtech SX1262 (LoRa)

- [lora-phy Rust crate](https://crates.io/crates/lora-phy)
- Frequency: 868.0 MHz (European SRD license-exempt band)

### SD SPI Storage

- Shares SPI2 with the LoRa radio.
- Mount point: `/sdcard/flight.csv` on a FAT filesystem

### Hobbywing Skywalker 12A ESC & DYS BX1306 (Airbag EDF)

- PWM Interface: 50 Hz RC servo signal (20,000 µs period)
	- Arming / Idle: 1000 µs pulse (0% throttle, motor stopped)
	- Inflation: 2000 µs pulse (100% throttle)
- Requires holding 1000 µs neutral for 2 seconds on startup before accepting motor commands.

## Launch stages

The CanSat rules require a descent from 1 km to ground in 77 seconds (~13 m/s descent), due to requiring a landing within 90 seconds.

1. Pre-launch:
	- On the launchpad
	- ESC armed in neutral (1000 µs)
	- Kalman filter initialized with the launchpad reference altitude and stationary barometric, accelerometric, and GPS measurements
2. Launch:
	- Launch phase detected when IMU acceleration exceeds 2.5 g
3. Descent:
	- Parachute deployed at apogee
	- Descent velocity monitored (~ 13 m/s)
4. Airbag deployment:
	- Fused Kalman-filter altitude estimate drops below 100 meters above ground level
	- ESC set to 100% throttle for 7 seconds to fully inflate the airbag cushion before ground impact
5. Landed:
	- Touchdown detected and the motor is shut down

## Building

### Prerequisites

- Rust toolchain with Xtensa support (`esp` toolchain)
- Espressif GCC/clang toolchain exported by `espup`
- `espflash` CLI tool (`cargo install espflash --locked`)

```bash
source "$HOME/export-esp.sh"
cargo build --release
```

```bash
espflash flash --monitor target/xtensa-esp32s3-espidf/release/cansat-flight-computer
```
