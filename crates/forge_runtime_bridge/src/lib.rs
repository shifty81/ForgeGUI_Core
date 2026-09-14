//! Authoring <-> runtime bridge used for PIE, Simulate, Play From Here,
//! detached play, and standalone-equivalent runtime sessions.
#![forbid(unsafe_code)]

use forge_authoring_core::AuthoringSession;
use forge_runtime_core::{InputSnapshot, RuntimeConfig, RuntimeEngine, RuntimeFrame, RuntimeMode};
use forge_scene_core::ForgeScene;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayMode {
    Simulate,
    PlayInEditor,
    PlayFromHere,
    Detached,
    Standalone,
    Headless,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayRequest {
    pub mode: PlayMode,
    pub start_position: Option<[f32; 3]>,
    pub preserve_runtime_changes: bool,
}

impl Default for PlayRequest {
    fn default() -> Self {
        Self {
            mode: PlayMode::PlayInEditor,
            start_position: None,
            preserve_runtime_changes: false,
        }
    }
}

#[derive(Default)]
pub struct RuntimeBridge {
    pub request: Option<PlayRequest>,
    pub runtime: RuntimeEngine,
    editor_snapshot: Option<ForgeScene>,
}

impl RuntimeBridge {
    pub fn play(
        &mut self,
        authoring: &AuthoringSession,
        request: PlayRequest,
    ) -> Result<(), String> {
        if self.runtime.scene.is_some() {
            self.stop(None)?;
        }

        self.editor_snapshot = Some(authoring.scene.clone());
        let runtime_scene = authoring.scene.runtime_clone();

        self.runtime.config = RuntimeConfig {
            mode: match request.mode {
                PlayMode::Headless => RuntimeMode::Headless,
                PlayMode::Simulate | PlayMode::PlayInEditor | PlayMode::PlayFromHere => {
                    RuntimeMode::EditorPreview
                }
                PlayMode::Detached | PlayMode::Standalone => RuntimeMode::Rendered,
            },
            ..RuntimeConfig::default()
        };

        self.runtime.start(runtime_scene)?;
        self.request = Some(request);
        Ok(())
    }

    pub fn update(&mut self, dt: f64, input: &InputSnapshot) -> Result<RuntimeFrame, String> {
        self.runtime.update(dt, input)
    }

    pub fn stop(&mut self, authoring: Option<&mut AuthoringSession>) -> Result<(), String> {
        let preserve = self
            .request
            .as_ref()
            .map(|request| request.preserve_runtime_changes)
            .unwrap_or(false);

        let runtime_scene = self.runtime.scene.clone();
        self.runtime.stop()?;

        if let Some(authoring) = authoring {
            if preserve {
                if let Some(scene) = runtime_scene {
                    authoring.scene = scene;
                    authoring.dirty = true;
                }
            } else if let Some(snapshot) = self.editor_snapshot.take() {
                authoring.scene = snapshot;
            }
        }

        self.request = None;
        Ok(())
    }

    pub fn pause(&mut self) {
        self.runtime.pause();
    }

    pub fn resume(&mut self) {
        self.runtime.resume();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_scene_core::ForgeScene;

    #[test]
    fn play_uses_runtime_clone() {
        let session = AuthoringSession::new("demo.document", ForgeScene::new("demo.scene", "Demo"));
        let mut bridge = RuntimeBridge::default();
        bridge.play(&session, PlayRequest::default()).unwrap();
        assert!(bridge.runtime.scene.is_some());
    }
}
