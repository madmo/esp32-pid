use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use embedded_hal::digital::v2::OutputPin;

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    let mut relay = peripherals.pins.gpio3.into_output().unwrap();

    let tsic_power = peripherals.pins.gpio1.into_output().unwrap();
    let tsic_data = peripherals.pins.gpio2.into_input().unwrap();
    let mut delay = esp_idf_hal::delay::Ets {};
    let mut sensor = tsic::Tsic::with_vdd_control(tsic::SensorType::Tsic306, tsic_data, tsic_power);

    let mut pid = pid_ctrl::PidCtrl::new_with_pid(0.7, 0.00005, 0.0);
    pid.init(95.0, 0.0);

    let mut last_measurement = std::time::Instant::now();
    loop {
        if let Ok(temperature) = sensor.read(&mut delay) {
            let output = pid.step(pid_ctrl::PidIn::new(temperature.as_celsius(), last_measurement.elapsed().as_millis() as f32));
            last_measurement = std::time::Instant::now();

            for i in 0..10 {
                let power_on = output.out > i as f32;

                if power_on {
                    relay.set_high().unwrap();
                } else {
                    relay.set_low().unwrap();
                }

                println!(
                    "# {};{};{};{}",
                    temperature.as_celsius(),
                    output.out,
                    i,
                    power_on
                );

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
