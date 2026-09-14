//! Shared semantic theming for ForgeGUI_Core.
#![forbid(unsafe_code)]

use forge_gui_core::{Rgba, ThemeTokens};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SurfaceRole {
    Sunken,
    Base,
    Raised1,
    Raised2,
    Raised3,
    Overlay,
    Focus,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct SurfaceTokens {
    pub fill: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub shadow_alpha: u8,
    pub corner_radius: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForgeTheme {
    pub id: String,
    pub label: String,
    pub base: ThemeTokens,
    pub density_scale: f32,
    pub motion_scale: f32,
}

impl ForgeTheme {
    pub fn forge_dark() -> Self {
        Self {
            id: "forge.theme.dark".into(),
            label: "Forge Dark".into(),
            base: ThemeTokens {
                background: Rgba(17, 20, 25, 255),
                panel: Rgba(23, 28, 35, 255),
                panel_raised: Rgba(31, 37, 45, 255),
                panel_recessed: Rgba(12, 15, 19, 255),
                border: Rgba(57, 67, 79, 255),
                border_focus: Rgba(72, 230, 161, 255),
                text: Rgba(229, 235, 242, 255),
                text_muted: Rgba(148, 160, 174, 255),
                accent: Rgba(72, 230, 161, 255),
                success: Rgba(73, 218, 145, 255),
                warning: Rgba(236, 183, 74, 255),
                danger: Rgba(240, 95, 109, 255),
                status_height: 26.0,
                toolbar_height: 40.0,
                corner_radius: 8.0,
                scrollbar_width: 11.0,
            },
            density_scale: 1.0,
            motion_scale: 1.0,
        }
    }

    pub fn graphite() -> Self {
        let mut theme = Self::forge_dark();
        theme.id = "forge.theme.graphite".into();
        theme.label = "Graphite".into();
        theme.base.background = Rgba(20, 20, 22, 255);
        theme.base.panel = Rgba(28, 28, 31, 255);
        theme.base.panel_raised = Rgba(37, 37, 41, 255);
        theme.base.accent = Rgba(117, 163, 255, 255);
        theme.base.border_focus = theme.base.accent;
        theme
    }

    pub fn high_contrast() -> Self {
        let mut theme = Self::forge_dark();
        theme.id = "forge.theme.high_contrast".into();
        theme.label = "High Contrast".into();
        theme.base.background = Rgba(8, 9, 11, 255);
        theme.base.panel = Rgba(15, 17, 20, 255);
        theme.base.panel_raised = Rgba(28, 31, 36, 255);
        theme.base.border = Rgba(98, 109, 123, 255);
        theme.base.text = Rgba(248, 250, 252, 255);
        theme
    }

    pub fn surface(&self, role: SurfaceRole) -> SurfaceTokens {
        let (fill, border, shadow_alpha) = match role {
            SurfaceRole::Sunken => (self.base.panel_recessed, self.base.border, 20),
            SurfaceRole::Base => (self.base.background, self.base.border, 0),
            SurfaceRole::Raised1 => (self.base.panel, self.base.border, 28),
            SurfaceRole::Raised2 => (self.base.panel_raised, self.base.border, 38),
            SurfaceRole::Raised3 => (lift(self.base.panel_raised, 8), self.base.border_focus, 48),
            SurfaceRole::Overlay => (lift(self.base.panel_raised, 12), self.base.border_focus, 72),
            SurfaceRole::Focus => (self.base.panel_raised, self.base.border_focus, 54),
        };

        SurfaceTokens {
            fill,
            border,
            text: self.base.text,
            text_muted: self.base.text_muted,
            accent: self.base.accent,
            shadow_alpha,
            corner_radius: self.base.corner_radius,
        }
    }
}

impl Default for ForgeTheme {
    fn default() -> Self {
        Self::forge_dark()
    }
}

fn lift(color: Rgba, amount: u8) -> Rgba {
    Rgba(
        color.0.saturating_add(amount),
        color.1.saturating_add(amount),
        color.2.saturating_add(amount),
        color.3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_surfaces_are_distinct() {
        let theme = ForgeTheme::forge_dark();
        assert_ne!(
            theme.surface(SurfaceRole::Raised1).fill,
            theme.surface(SurfaceRole::Raised2).fill
        );
    }

    #[test]
    fn high_contrast_keeps_focus_accent() {
        let theme = ForgeTheme::high_contrast();
        assert_eq!(theme.base.border_focus, theme.base.accent);
    }
}
