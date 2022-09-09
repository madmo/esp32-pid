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

    let window_size = 12;
    let mut pid = pid::Pid::new(67.0, 67.0 / 399.0, 67.0 * 399.0, 100.0, 100.0, 100.0, window_size as f32, 95.0);
    
    let mut counter = 0;
    let mut duty = 0.0;

    loop {
        let on = if duty < counter as f32 {
            relay.set_low().unwrap();
            false
        } else {
            relay.set_high().unwrap();
            true
        };

        if let Ok(temperature) = sensor.read(&mut delay) {
            let output = pid.next_control_output(temperature.as_celsius());
            duty = output.output;

            println!(
                "# {};{};{};{}",
                on,
                temperature.as_celsius(),
                output.output,
                counter
            );
        }

        counter += 1;
        if counter > window_size {
            counter = 0;
        }
    }
}
