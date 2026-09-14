//! Headless-capable shared runtime engine foundation.
#![forbid(unsafe_code)]

use forge_scene_core::ForgeScene;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeMode {
    Headless,
    Rendered,
    EditorPreview,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeState {
    Stopped,
    Starting,
    Running,
    Paused,
    ShuttingDown,
    Failed,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct InputSnapshot {
    pub actions_down: BTreeSet<String>,
    pub actions_pressed: BTreeSet<String>,
    pub actions_released: BTreeSet<String>,
    pub pointer_position: Option<[f32; 2]>,
    pub pointer_delta: [f32; 2],
    pub scroll_delta: [f32; 2],
    pub axes: BTreeMap<String, f32>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct FixedStepClock {
    pub step_seconds: f64,
    pub accumulator: f64,
    pub simulation_time: f64,
    pub tick: u64,
    pub max_substeps: u32,
}

impl Default for FixedStepClock {
    fn default() -> Self {
        Self {
            step_seconds: 1.0 / 60.0,
            accumulator: 0.0,
            simulation_time: 0.0,
            tick: 0,
            max_substeps: 8,
        }
    }
}

impl FixedStepClock {
    pub fn advance(&mut self, frame_seconds: f64) -> u32 {
        self.accumulator += frame_seconds.max(0.0);
        let mut steps = 0;

        while self.accumulator + f64::EPSILON >= self.step_seconds && steps < self.max_substeps {
            self.accumulator -= self.step_seconds;
            self.simulation_time += self.step_seconds;
            self.tick += 1;
            steps += 1;
        }

        steps
    }

    pub fn interpolation_alpha(&self) -> f64 {
        if self.step_seconds <= f64::EPSILON {
            0.0
        } else {
            (self.accumulator / self.step_seconds).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeConfig {
    pub mode: RuntimeMode,
    pub fixed_step_seconds: f64,
    pub max_substeps: u32,
    pub pause_when_unfocused: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            mode: RuntimeMode::Rendered,
            fixed_step_seconds: 1.0 / 60.0,
            max_substeps: 8,
            pause_when_unfocused: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeFrame {
    pub frame_seconds: f64,
    pub fixed_steps: u32,
    pub interpolation_alpha: f64,
    pub tick: u64,
    pub simulation_time: f64,
}

pub trait RuntimeSystem: Send {
    fn id(&self) -> &'static str;
    fn start(&mut self, _scene: &mut ForgeScene) -> Result<(), String> {
        Ok(())
    }
    fn fixed_update(
        &mut self,
        _scene: &mut ForgeScene,
        _input: &InputSnapshot,
        _dt: f64,
    ) -> Result<(), String> {
        Ok(())
    }
    fn frame_update(
        &mut self,
        _scene: &mut ForgeScene,
        _input: &InputSnapshot,
        _dt: f64,
        _alpha: f64,
    ) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self, _scene: &mut ForgeScene) -> Result<(), String> {
        Ok(())
    }
}

pub struct RuntimeEngine {
    pub config: RuntimeConfig,
    pub state: RuntimeState,
    pub scene: Option<ForgeScene>,
    pub clock: FixedStepClock,
    systems: Vec<Box<dyn RuntimeSystem>>,
}

impl RuntimeEngine {
    pub fn new(config: RuntimeConfig) -> Self {
        let clock = FixedStepClock {
            step_seconds: config.fixed_step_seconds,
            max_substeps: config.max_substeps,
            ..FixedStepClock::default()
        };
        Self {
            config,
            state: RuntimeState::Stopped,
            scene: None,
            clock,
            systems: Vec::new(),
        }
    }

    pub fn register_system(&mut self, system: Box<dyn RuntimeSystem>) {
        self.systems.push(system);
    }

    pub fn start(&mut self, mut scene: ForgeScene) -> Result<(), String> {
        scene
            .validate()
            .map_err(|error| format!("scene validation failed: {error}"))?;
        self.state = RuntimeState::Starting;

        for system in &mut self.systems {
            system.start(&mut scene)?;
        }

        self.scene = Some(scene);
        self.clock = FixedStepClock {
            step_seconds: self.config.fixed_step_seconds,
            max_substeps: self.config.max_substeps,
            ..FixedStepClock::default()
        };
        self.state = RuntimeState::Running;
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == RuntimeState::Running {
            self.state = RuntimeState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == RuntimeState::Paused {
            self.state = RuntimeState::Running;
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.state = RuntimeState::ShuttingDown;
        if let Some(scene) = &mut self.scene {
            for system in self.systems.iter_mut().rev() {
                system.stop(scene)?;
            }
        }
        self.scene = None;
        self.state = RuntimeState::Stopped;
        Ok(())
    }

    pub fn update(
        &mut self,
        frame_seconds: f64,
        input: &InputSnapshot,
    ) -> Result<RuntimeFrame, String> {
        if self.state == RuntimeState::Paused {
            return Ok(RuntimeFrame {
                frame_seconds,
                fixed_steps: 0,
                interpolation_alpha: self.clock.interpolation_alpha(),
                tick: self.clock.tick,
                simulation_time: self.clock.simulation_time,
            });
        }
        if self.state != RuntimeState::Running {
            return Err("runtime is not running".into());
        }

        let scene = self
            .scene
            .as_mut()
            .ok_or_else(|| "runtime has no scene".to_string())?;
        let fixed_steps = self.clock.advance(frame_seconds);

        for _ in 0..fixed_steps {
            for system in &mut self.systems {
                system.fixed_update(scene, input, self.clock.step_seconds)?;
            }
        }

        let alpha = self.clock.interpolation_alpha();
        for system in &mut self.systems {
            system.frame_update(scene, input, frame_seconds, alpha)?;
        }

        Ok(RuntimeFrame {
            frame_seconds,
            fixed_steps,
            interpolation_alpha: alpha,
            tick: self.clock.tick,
            simulation_time: self.clock.simulation_time,
        })
    }
}

impl Default for RuntimeEngine {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_step_clock_caps_substeps() {
        let mut clock = FixedStepClock {
            step_seconds: 0.1,
            max_substeps: 2,
            ..FixedStepClock::default()
        };
        assert_eq!(clock.advance(1.0), 2);
    }

    #[test]
    fn headless_runtime_starts_valid_scene() {
        let config = RuntimeConfig {
            mode: RuntimeMode::Headless,
            ..RuntimeConfig::default()
        };
        let mut runtime = RuntimeEngine::new(config);
        runtime
            .start(ForgeScene::new("demo.scene", "Demo"))
            .unwrap();
        assert_eq!(runtime.state, RuntimeState::Running);
    }
}
