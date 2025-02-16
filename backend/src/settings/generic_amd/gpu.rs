use std::sync::Mutex;
use ryzenadj_rs::RyzenAccess;

use crate::persist::GpuJson;
use crate::settings::MinMax;
use crate::settings::generic::Gpu as GenericGpu;
use crate::settings::{OnResume, OnSet, SettingError, SettingVariant};
use crate::settings::TGpu;

fn ryzen_adj_or_log() -> Option<Mutex<RyzenAccess>> {
    match RyzenAccess::new() {
        Ok(x) => Some(Mutex::new(x)),
        Err(e) => {
            log::error!("RyzenAdj init error: {}", e);
            None
        }
    }
}

#[derive(Debug)]
pub struct Gpu {
    generic: GenericGpu,
    implementor: Option<Mutex<RyzenAccess>>,
    state: crate::state::generic::Gpu, // NOTE this is re-used for simplicity
}

impl Gpu {
    pub fn from_limits(limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            generic: GenericGpu::from_limits(limits),
            implementor: ryzen_adj_or_log(),
            state: Default::default(),
        }
    }

    pub fn from_json_and_limits(other: GpuJson, version: u64, limits: limits_core::json::GenericGpuLimit) -> Self {
        Self {
            generic: GenericGpu::from_json_and_limits(other, version, limits),
            implementor: ryzen_adj_or_log(),
            state: Default::default(),
        }
    }

    fn set_all(&mut self) -> Result<(), Vec<SettingError>> {
        let mutex = match &self.implementor {
            Some(x) => x,
            None => {
                return Err(vec![SettingError {
                    msg: "RyzenAdj unavailable".to_owned(),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let lock = match mutex.lock() {
            Ok(x) => x,
            Err(e) => {
                return Err(vec![SettingError {
                    msg: format!("RyzenAdj lock acquire failed: {}", e),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let mut errors = Vec::new();
        if let Some(fast_ppt) = &self.generic.fast_ppt {
            if self.state.old_fast_ppt.is_none() {
                self.state.old_fast_ppt = Some(lock.get_fast_value() as _);
            }
            lock.set_fast_limit(*fast_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        } else if let Some(fast_ppt) = &self.state.old_fast_ppt {
            lock.set_fast_limit(*fast_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
            self.state.old_fast_ppt = None;
        }
        if let Some(slow_ppt) = &self.generic.slow_ppt {
            if self.state.old_slow_ppt.is_none() {
                self.state.old_slow_ppt = Some(lock.get_slow_value() as _);
            }
            lock.set_slow_limit(*slow_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        } else if let Some(slow_ppt) = &self.state.old_slow_ppt {
            lock.set_slow_limit(*slow_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
            self.state.old_slow_ppt = None;
        }
        if let Some(clock_limits) = &self.generic.clock_limits {
            self.state.clock_limits_set = true;
            lock.set_max_gfxclk_freq(clock_limits.max as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_max_gfxclk_freq({}) err: {}", clock_limits.max, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
            lock.set_min_gfxclk_freq(clock_limits.min as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_min_gfxclk_freq({}) err: {}", clock_limits.min, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        } else if self.state.clock_limits_set {
            self.state.clock_limits_set = false;
            let limits = self.generic.limits();
            if let Some(min_limits) = limits.clock_min_limits {
                if let Some(max_limits) = limits.clock_max_limits {
                    lock.set_max_gfxclk_freq(max_limits.max as _).map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_max_gfxclk_freq({}) err: {}", max_limits.max, e),
                        setting: SettingVariant::Gpu,
                    }).unwrap_or_else(|e| errors.push(e));
                    lock.set_min_gfxclk_freq(min_limits.min as _).map_err(|e| SettingError {
                        msg: format!("RyzenAdj set_min_gfxclk_freq({}) err: {}", min_limits.min, e),
                        setting: SettingVariant::Gpu,
                    }).unwrap_or_else(|e| errors.push(e));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn resume_all(&self) -> Result<(), Vec<SettingError>> {
        // like set_all() but without updating state
        // -- assumption: state is already up to date
        let mutex = match &self.implementor {
            Some(x) => x,
            None => {
                return Err(vec![SettingError {
                    msg: "RyzenAdj unavailable".to_owned(),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let lock = match mutex.lock() {
            Ok(x) => x,
            Err(e) => {
                return Err(vec![SettingError {
                    msg: format!("RyzenAdj lock acquire failed: {}", e),
                    setting: SettingVariant::Gpu,
                }]);
            }
        };
        let mut errors = Vec::new();
        if let Some(fast_ppt) = &self.generic.fast_ppt {
            lock.set_fast_limit(*fast_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_fast_limit({}) err: {}", *fast_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        }
        if let Some(slow_ppt) = &self.generic.slow_ppt {
            lock.set_slow_limit(*slow_ppt as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_slow_limit({}) err: {}", *slow_ppt, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        }
        if let Some(clock_limits) = &self.generic.clock_limits {
            lock.set_max_gfxclk_freq(clock_limits.max as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_max_gfxclk_freq({}) err: {}", clock_limits.max, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
            lock.set_min_gfxclk_freq(clock_limits.min as _).map_err(|e| SettingError {
                msg: format!("RyzenAdj set_min_gfxclk_freq({}) err: {}", clock_limits.min, e),
                setting: SettingVariant::Gpu,
            }).unwrap_or_else(|e| errors.push(e));
        }
        Ok(())
    }
}

impl OnResume for Gpu {
    fn on_resume(&self) -> Result<(), Vec<SettingError>> {
        self.generic.on_resume()?;
        self.resume_all()
    }
}

impl OnSet for Gpu {
    fn on_set(&mut self) -> Result<(), Vec<SettingError>> {
        self.generic.on_set()?;
        self.set_all()
    }
}

impl TGpu for Gpu {
    fn limits(&self) -> crate::api::GpuLimits {
        self.generic.limits()
    }

    fn json(&self) -> crate::persist::GpuJson {
        self.generic.json()
    }

    fn ppt(&mut self, fast: Option<u64>, slow: Option<u64>) {
        self.generic.ppt(fast, slow)
    }

    fn get_ppt(&self) -> (Option<u64>, Option<u64>) {
        self.generic.get_ppt()
    }

    fn clock_limits(&mut self, limits: Option<MinMax<u64>>) {
        self.generic.clock_limits(limits)
    }

    fn get_clock_limits(&self) -> Option<&MinMax<u64>> {
        self.generic.get_clock_limits()
    }

    fn slow_memory(&mut self) -> &mut bool {
        self.generic.slow_memory()
    }

    fn provider(&self) -> crate::persist::DriverJson {
        crate::persist::DriverJson::GenericAMD
    }
}
