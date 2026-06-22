use imu_core::{ImuCommand, ImuReading, SetParamsCmd};
use imu_transport::{Device, Transport};
use tokio::sync::mpsc;

use crate::types::*;

/// Background task that handles all device communication
pub async fn background_task<T: Transport + 'static>(
    device: Device<T>,
    mut command_rx: mpsc::Receiver<DeviceCommand>,
    data_tx: mpsc::Sender<ImuReading>,
) {
    loop {
        // Wait for a command or timeout to check for data
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    DeviceCommand::Connect(_params) => {
                        if let Err(e) = device.connect().await {
                            eprintln!("Connect failed: {}", e);
                        }
                    }
                    DeviceCommand::Disconnect => {
                        if let Err(e) = device.disconnect().await {
                            eprintln!("Disconnect failed: {}", e);
                        }
                        break;
                    }
                    DeviceCommand::SetConfig(config) => {
                        let params = SetParamsCmd {
                            still_threshold: 5,
                            still_zero_speed: 255,
                            move_zero_speed: 0,
                            compass_on: true,
                            barometer_filter: config.filter_level.min(3),
                            fps: config.report_rate.min(255) as u8,
                            gyro_filter: config.filter_level,
                            accel_filter: config.filter_level,
                            compass_filter: config.filter_level,
                            subscription_tag: 0x0FFF, // Enable all channels
                        };
                        
                        let cmd = ImuCommand::SetParams(params);
                        if let Err(e) = device.send_command(&cmd).await {
                            eprintln!("SetConfig failed: {}", e);
                        }
                    }
                    DeviceCommand::Calibrate(cal_type) => {
                        let cmd = match cal_type {
                            CalibrationType::Accelerometer => ImuCommand::SimpleAccelCalibration,
                            CalibrationType::Gyroscope => ImuCommand::ZeroWorldXYZ,
                            CalibrationType::Magnetometer => ImuCommand::StopCompassCalibration,
                        };
                        if let Err(e) = device.send_command(&cmd).await {
                            eprintln!("Calibrate failed: {}", e);
                        }
                    }
                    DeviceCommand::QueryStatus => {
                        let cmd = ImuCommand::QueryStatus;
                        match device.send_command(&cmd).await {
                            Ok(response) => {
                                if let imu_core::ImuResponse::DeviceStatus(status) = response {
                                    println!("Device Status:");
                                    println!("  Battery: {}% ({}mV)", status.battery_level, status.battery_voltage_mv);
                                    println!("  Firmware: {}", status.firmware_version);
                                    println!("  Model: {}", status.product_model);
                                    println!("  FPS: {}Hz", status.fps);
                                    println!("  Auto Report: {}", if status.auto_report_on { "ON" } else { "OFF" });
                                }
                            }
                            Err(e) => {
                                eprintln!("QueryStatus failed: {}", e);
                            }
                        }
                    }
                    DeviceCommand::StartAutoReport => {
                        let cmd = ImuCommand::StartAutoReport;
                        if let Err(e) = device.send_command(&cmd).await {
                            eprintln!("StartAutoReport failed: {}", e);
                        }
                    }
                    DeviceCommand::StopAutoReport => {
                        let cmd = ImuCommand::StopAutoReport;
                        if let Err(e) = device.send_command(&cmd).await {
                            eprintln!("StopAutoReport failed: {}", e);
                        }
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(1)) => {
                // Try to receive data
                match device.try_receive_data().await {
                    Ok(Some(reading)) => {
                        // Custom backpressure: if channel is full, drop oldest
                        if data_tx.capacity() == 0 {
                            // Channel is full, we can't drop from receiver side
                            // Just skip this reading
                        }
                        if data_tx.try_send(reading).is_err() {
                            // Channel full or closed, skip this reading
                        }
                    }
                    Ok(None) => {
                        // No data or non-data response
                    }
                    Err(e) => {
                        eprintln!("Receive error: {}", e);
                    }
                }
            }
        }
    }
}
