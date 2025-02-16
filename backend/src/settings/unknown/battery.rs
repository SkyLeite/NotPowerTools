use std::convert::Into;

use crate::settings::{OnResume, OnSet, SettingError};
use crate::settings::TBattery;
use crate::persist::BatteryJson;

#[derive(Debug, Clone)]
pub struct Battery;

impl Into<BatteryJson> for Battery {
    #[inline]
    fn into(self) -> BatteryJson {
        BatteryJson {
            charge_rate: None,
            charge_mode: None,
        }
    }
}

impl OnSet for Battery {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl OnResume for Battery {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        Ok(())
    }
}

impl TBattery for Battery {
    fn limits(&self) -> crate::api::BatteryLimits {
        crate::api::BatteryLimits {
            charge_current: None,
            charge_current_step: 50,
            charge_modes: vec![],
        }
    }

    fn json(&self) -> crate::persist::BatteryJson {
        self.clone().into()
    }

    fn charge_rate(&mut self, _rate: Option<u64>) {
    }

    fn get_charge_rate(&self) -> Option<u64> {
        None
    }

    fn charge_mode(&mut self, _rate: Option<String>) {
    }

    fn get_charge_mode(&self) -> Option<String> {
        None
    }

    fn read_charge_full(&self) -> Option<f64> { None }

    fn read_charge_now(&self) -> Option<f64> { None }

    fn read_charge_design(&self) -> Option<f64> { None }

    fn read_current_now(&self) -> Option<f64> { None }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::Unknown
    }
}
