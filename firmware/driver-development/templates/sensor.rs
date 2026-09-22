// Initial integration template: dummy data only, no hardware transactions.
use serde_json::{json, Value};

use super::types::{
    copy_config_into_partition, getters, SensorDriver, SensorDriverGeneralConfiguration,
    SENSOR_SETTINGS_PARTITION_SIZE,
};

pub const TYPE_NAME: &str = "example_sensor";

// Special settings: byte 0 is version 1; the remaining bytes are reserved.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ExampleSensorSpecialConfiguration {
    bytes: [u8; SENSOR_SETTINGS_PARTITION_SIZE],
}

impl ExampleSensorSpecialConfiguration {
    pub fn parse_from_values(_values: Value) -> Result<Self, &'static str> {
        let mut bytes = [0; SENSOR_SETTINGS_PARTITION_SIZE];
        bytes[0] = 1;
        Ok(Self { bytes })
    }

    pub fn new_from_bytes(bytes: [u8; SENSOR_SETTINGS_PARTITION_SIZE]) -> Self {
        Self { bytes }
    }

    fn is_valid(&self) -> bool {
        self.bytes[0] == 1 && self.bytes[1..].iter().all(|byte| *byte == 0)
    }
}

pub struct ExampleSensor {
    general_config: SensorDriverGeneralConfiguration,
    special_config: ExampleSensorSpecialConfiguration,
    sample: u8,
    measured_value: Option<f64>,
}

impl ExampleSensor {
    pub fn new(
        general_config: SensorDriverGeneralConfiguration,
        special_config: ExampleSensorSpecialConfiguration,
    ) -> Self {
        Self {
            general_config,
            special_config,
            sample: 0,
            measured_value: None,
        }
    }
}

impl SensorDriver for ExampleSensor {
    getters!();

    fn setup(&mut self, _board: &mut dyn rriv_board::RRIVBoard) {
        self.sample = 0;
        self.measured_value = None;
        // Step 4: device initialization and declared resource setup.
    }

    fn take_measurement(&mut self, _board: &mut dyn rriv_board::RRIVBoard) {
        self.measured_value = None;
        if !self.special_config.is_valid() {
            return;
        }
        // Step 5: replace this counter with an actual sensor transaction.
        self.sample = self.sample % 10 + 1;
        self.measured_value = Some(f64::from(self.sample));
    }

    fn get_measured_parameter_count(&mut self) -> usize {
        1
    }

    fn get_measured_parameter_value(&mut self, index: usize) -> Result<f64, ()> {
        if index == 0 {
            self.measured_value.ok_or(())
        } else {
            Err(())
        }
    }

    fn get_measured_parameter_identifier(&mut self, index: usize) -> [u8; 16] {
        let mut label = [0; 16];
        let text: &[u8] = if index == 0 { b"dummy" } else { b"invalid" };
        label[..text.len()].copy_from_slice(text);
        label
    }

    fn get_configuration_json(&mut self) -> Value {
        let mut id_bytes = self.general_config.id;
        let id = util::str_from_utf8(&mut id_bytes).unwrap_or_default();
        json!({
            "id": id,
            "type": TYPE_NAME,
            "dummy_data": true,
            "config_valid": self.special_config.is_valid()
        })
    }

    fn update(&mut self, _values: Value) -> Result<(), &'static str> {
        // Step 7: validate and apply device-specific settings.
        Err("configuration updates not implemented")
    }

    // Step 6 expands storage/getters to match the sensor's real outputs.
    // Step 8 adds calibration if planned.
    // Default GPIO requests are empty; actuator implementation is deferred.
}
