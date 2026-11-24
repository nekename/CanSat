use crate::DataFrame;

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};

pub const FLIGHT_CSV_PATH: &str = "/sdcard/flight.csv";
pub const FLUSH_INTERVAL_FRAMES: usize = 16;

pub const CSV_HEADER: &str = "temperature_c,pressure_pa,magnetic_x_mgauss,magnetic_y_mgauss,magnetic_z_mgauss,\
accel_x_g,accel_y_g,accel_z_g,gyro_x_mdps,gyro_y_mdps,gyro_z_mdps,\
latitude_deg,longitude_deg,gps_altitude_m,satellites,speed_mps,course_deg,fix_valid\n";

fn append_number(value: Option<impl std::fmt::Display>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

pub fn frame_csv_row(frame: &DataFrame) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
        frame.bmp.temperature_c,
        frame.bmp.pressure_pa,
        frame.lis.magnetic_x_mgauss,
        frame.lis.magnetic_y_mgauss,
        frame.lis.magnetic_z_mgauss,
        frame.imu.accel_x_g,
        frame.imu.accel_y_g,
        frame.imu.accel_z_g,
        frame.imu.gyro_x_mdps,
        frame.imu.gyro_y_mdps,
        frame.imu.gyro_z_mdps,
        append_number(frame.gps.latitude_deg),
        append_number(frame.gps.longitude_deg),
        append_number(frame.gps.altitude_m),
        append_number(frame.gps.satellites),
        append_number(frame.gps.speed_mps),
        append_number(frame.gps.course_deg),
        frame.gps.fix_valid,
    )
}

pub fn ensure_header(path: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    file.seek(SeekFrom::Start(0))?;
    let mut prefix = [0u8; 1];
    if file.read(&mut prefix)? == 0 {
        file.write_all(CSV_HEADER.as_bytes())?;
    }
    Ok(())
}

pub struct FlightCsvWriter {
    file: BufWriter<File>,
    frames_since_flush: usize,
}

impl FlightCsvWriter {
    pub fn open(path: &str) -> std::io::Result<Self> {
        ensure_header(path)?;
        let file = OpenOptions::new().append(true).open(path)?;
        Ok(Self {
            file: BufWriter::new(file),
            frames_since_flush: 0,
        })
    }

    pub fn open_flight_csv() -> std::io::Result<Self> {
        Self::open(FLIGHT_CSV_PATH)
    }

    pub fn append_frame(&mut self, frame: &DataFrame) -> std::io::Result<()> {
        self.file.write_all(frame_csv_row(frame).as_bytes())?;
        self.frames_since_flush += 1;
        if self.frames_since_flush == FLUSH_INTERVAL_FRAMES {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        self.frames_since_flush = 0;
        Ok(())
    }
}

impl Drop for FlightCsvWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
