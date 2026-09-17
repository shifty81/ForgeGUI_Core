//! Shared semantic theming for ForgeGUI_Core.
#![forbid(unsafe_code)]

pub use forge_gui_core::Rgba;
use forge_gui_core::ThemeTokens;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForgeThemePreset {
    #[default]
    ForgeDark,
    MidnightMint,
    Graphite,
    WarmEmber,
    HighContrast,
}

impl ForgeThemePreset {
    pub const ALL: [Self; 5] = [
        Self::ForgeDark,
        Self::MidnightMint,
        Self::Graphite,
        Self::WarmEmber,
        Self::HighContrast,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::ForgeDark => "Forge Dark",
            Self::MidnightMint => "Midnight Mint",
            Self::Graphite => "Graphite",
            Self::WarmEmber => "Warm Ember",
            Self::HighContrast => "High Contrast",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForgeDensity {
    Compact,
    #[default]
    Standard,
    Comfortable,
}

impl ForgeDensity {
    pub const ALL: [Self; 3] = [Self::Compact, Self::Standard, Self::Comfortable];

    pub const fn scale(self) -> f32 {
        match self {
            Self::Compact => 0.90,
            Self::Standard => 1.0,
            Self::Comfortable => 1.12,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Standard => "Standard",
            Self::Comfortable => "Comfortable",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum HighlightMode {
    Minimal,
    Outline,
    #[default]
    Tint,
    Fill,
    AccentBar,
    Glow,
}

impl HighlightMode {
    pub const ALL: [Self; 6] = [
        Self::Minimal,
        Self::Outline,
        Self::Tint,
        Self::Fill,
        Self::AccentBar,
        Self::Glow,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimal => "Minimal",
            Self::Outline => "Outline",
            Self::Tint => "Tint",
            Self::Fill => "Fill",
            Self::AccentBar => "Accent bar",
            Self::Glow => "Glow",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct InteractionTokens {
    pub hover: HighlightMode,
    pub pressed: HighlightMode,
    pub selected: HighlightMode,
    pub focus: HighlightMode,
    pub surface_radius: f32,
    pub control_radius: f32,
    pub popup_radius: f32,
    pub border_width: f32,
    pub focus_width: f32,
    pub disabled_alpha: f32,
}

impl Default for InteractionTokens {
    fn default() -> Self {
        Self {
            hover: HighlightMode::Tint,
            pressed: HighlightMode::Fill,
            selected: HighlightMode::AccentBar,
            focus: HighlightMode::Outline,
            surface_radius: 7.0,
            control_radius: 5.0,
            popup_radius: 9.0,
            border_width: 1.0,
            focus_width: 1.5,
            disabled_alpha: 0.45,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForgeMetrics {
    pub product_bar_height: f32,
    pub menu_bar_height: f32,
    pub action_bar_height: f32,
    pub workspace_tab_height: f32,
    pub panel_header_height: f32,
    pub asset_row_height: f32,
    pub property_row_height: f32,
    pub bottom_tab_height: f32,
    pub status_bar_height: f32,
    pub splitter_width: f32,
    pub icon_size: f32,
    pub panel_padding: i8,
    pub outer_gap: f32,
}

impl Default for ForgeMetrics {
    fn default() -> Self {
        Self {
            product_bar_height: 32.0,
            menu_bar_height: 27.0,
            action_bar_height: 32.0,
            workspace_tab_height: 28.0,
            panel_header_height: 27.0,
            asset_row_height: 26.0,
            property_row_height: 27.0,
            bottom_tab_height: 26.0,
            status_bar_height: 27.0,
            splitter_width: 3.0,
            icon_size: 15.0,
            panel_padding: 5,
            outer_gap: 2.0,
        }
    }
}

impl ForgeMetrics {
    pub fn scaled(self, factor: f32) -> Self {
        let factor = factor.max(0.5);
        Self {
            product_bar_height: self.product_bar_height * factor,
            menu_bar_height: self.menu_bar_height * factor,
            action_bar_height: self.action_bar_height * factor,
            workspace_tab_height: self.workspace_tab_height * factor,
            panel_header_height: self.panel_header_height * factor,
            asset_row_height: self.asset_row_height * factor,
            property_row_height: self.property_row_height * factor,
            bottom_tab_height: self.bottom_tab_height * factor,
            status_bar_height: self.status_bar_height * factor,
            splitter_width: self.splitter_width * factor,
            icon_size: self.icon_size * factor,
            panel_padding: self.panel_padding,
            outer_gap: self.outer_gap * factor,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChromeTokens {
    pub shell: Rgba,
    pub product_bar: Rgba,
    pub menu_bar: Rgba,
    pub action_bar: Rgba,
    pub workspace_tabs: Rgba,
    pub panel_header: Rgba,
    pub bottom_tray: Rgba,
    pub status_bar: Rgba,
    pub canvas: Rgba,
    pub separator: Rgba,
}

impl Default for ChromeTokens {
    fn default() -> Self {
        Self {
            shell: Rgba(13, 15, 18, 255),
            product_bar: Rgba(19, 22, 26, 255),
            menu_bar: Rgba(23, 26, 31, 255),
            action_bar: Rgba(27, 31, 36, 255),
            workspace_tabs: Rgba(18, 21, 25, 255),
            panel_header: Rgba(31, 35, 41, 255),
            bottom_tray: Rgba(19, 22, 26, 255),
            status_bar: Rgba(15, 17, 20, 255),
            canvas: Rgba(11, 13, 16, 255),
            separator: Rgba(49, 56, 65, 255),
        }
    }
}

impl ChromeTokens {
    fn forge_dark(base: &ThemeTokens) -> Self {
        Self {
            shell: Rgba(13, 15, 18, 255),
            product_bar: Rgba(19, 22, 26, 255),
            menu_bar: Rgba(23, 26, 31, 255),
            action_bar: Rgba(27, 31, 36, 255),
            workspace_tabs: Rgba(18, 21, 25, 255),
            panel_header: Rgba(31, 35, 41, 255),
            bottom_tray: Rgba(19, 22, 26, 255),
            status_bar: Rgba(15, 17, 20, 255),
            canvas: base.panel_recessed,
            separator: Rgba(49, 56, 65, 255),
        }
    }
}

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
    #[serde(default)]
    pub density: ForgeDensity,
    #[serde(default)]
    pub metrics: ForgeMetrics,
    #[serde(default)]
    pub chrome: ChromeTokens,
    #[serde(default)]
    pub interaction: InteractionTokens,
}

impl ForgeTheme {
    pub fn from_preset(preset: ForgeThemePreset) -> Self {
        match preset {
            ForgeThemePreset::ForgeDark => Self::forge_dark(),
            ForgeThemePreset::MidnightMint => Self::midnight_mint(),
            ForgeThemePreset::Graphite => Self::graphite(),
            ForgeThemePreset::WarmEmber => Self::warm_ember(),
            ForgeThemePreset::HighContrast => Self::high_contrast(),
        }
    }

    pub fn forge_dark() -> Self {
        let base = ThemeTokens {
            background: Rgba(16, 18, 22, 255),
            panel: Rgba(24, 27, 32, 255),
            panel_raised: Rgba(33, 37, 43, 255),
            panel_recessed: Rgba(11, 13, 16, 255),
            border: Rgba(49, 56, 65, 255),
            border_focus: Rgba(72, 230, 161, 255),
            text: Rgba(229, 235, 242, 255),
            text_muted: Rgba(148, 160, 174, 255),
            accent: Rgba(72, 230, 161, 255),
            success: Rgba(73, 218, 145, 255),
            warning: Rgba(236, 183, 74, 255),
            danger: Rgba(240, 95, 109, 255),
            status_height: 26.0,
            toolbar_height: 40.0,
            corner_radius: 7.0,
            scrollbar_width: 11.0,
        };

        Self {
            id: "forge.theme.dark".into(),
            label: "Forge Dark".into(),
            chrome: ChromeTokens::forge_dark(&base),
            base,
            density_scale: 1.0,
            motion_scale: 1.0,
            density: ForgeDensity::Standard,
            metrics: ForgeMetrics::default(),
            interaction: InteractionTokens::default(),
        }
    }

    pub fn midnight_mint() -> Self {
        let mut theme = Self::forge_dark();
        theme.id = "forge.theme.midnight_mint".into();
        theme.label = "Midnight Mint".into();
        theme.base.background = Rgba(13, 16, 18, 255);
        theme.base.panel = Rgba(20, 25, 28, 255);
        theme.base.panel_raised = Rgba(29, 36, 39, 255);
        theme.base.panel_recessed = Rgba(8, 11, 13, 255);
        theme.base.accent = Rgba(91, 238, 181, 255);
        theme.base.border_focus = theme.base.accent;
        theme.chrome = ChromeTokens::forge_dark(&theme.base);
        theme
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
        theme.chrome = ChromeTokens::forge_dark(&theme.base);
        theme
    }

    pub fn warm_ember() -> Self {
        let mut theme = Self::forge_dark();
        theme.id = "forge.theme.warm_ember".into();
        theme.label = "Warm Ember".into();
        theme.base.background = Rgba(18, 16, 16, 255);
        theme.base.panel = Rgba(29, 25, 24, 255);
        theme.base.panel_raised = Rgba(41, 34, 31, 255);
        theme.base.panel_recessed = Rgba(12, 10, 10, 255);
        theme.base.accent = Rgba(242, 156, 92, 255);
        theme.base.border_focus = theme.base.accent;
        theme.chrome = ChromeTokens::forge_dark(&theme.base);
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
        theme.chrome = ChromeTokens::forge_dark(&theme.base);
        theme
    }

    pub fn effective_metrics(&self) -> ForgeMetrics {
        self.metrics
            .scaled(self.density.scale() * self.density_scale.max(0.5))
    }

    pub fn hover_fill(&self) -> Rgba {
        interaction_fill(self.base.panel, self.base.accent, self.interaction.hover)
    }

    pub fn pressed_fill(&self) -> Rgba {
        interaction_fill(
            self.base.panel_raised,
            self.base.accent,
            self.interaction.pressed,
        )
    }

    pub fn selected_fill(&self) -> Rgba {
        interaction_fill(
            self.base.panel_raised,
            self.base.accent,
            self.interaction.selected,
        )
    }

    pub fn interaction_stroke(&self, mode: HighlightMode) -> Rgba {
        match mode {
            HighlightMode::Minimal | HighlightMode::Tint => self.base.border,
            HighlightMode::Outline
            | HighlightMode::Fill
            | HighlightMode::AccentBar
            | HighlightMode::Glow => self.base.accent,
        }
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

fn interaction_fill(base: Rgba, accent: Rgba, mode: HighlightMode) -> Rgba {
    let amount = match mode {
        HighlightMode::Minimal | HighlightMode::Outline => 0.0,
        HighlightMode::AccentBar => 0.08,
        HighlightMode::Tint => 0.14,
        HighlightMode::Glow => 0.22,
        HighlightMode::Fill => 0.30,
    };
    blend(base, accent, amount)
}

fn blend(a: Rgba, b: Rgba, amount: f32) -> Rgba {
    let amount = amount.clamp(0.0, 1.0);
    let mix = |left: u8, right: u8| {
        (left as f32 + (right as f32 - left as f32) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgba(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2), a.3)
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

    #[test]
    fn compact_density_reduces_creator_metrics() {
        let mut theme = ForgeTheme::forge_dark();
        let standard = theme.effective_metrics();
        theme.density = ForgeDensity::Compact;
        let compact = theme.effective_metrics();
        assert!(compact.action_bar_height < standard.action_bar_height);
        assert!(compact.asset_row_height < standard.asset_row_height);
    }

    #[test]
    fn interaction_tokens_are_valid() {
        let theme = ForgeTheme::forge_dark();
        assert!(theme.interaction.surface_radius >= 0.0);
        assert!(theme.interaction.control_radius >= 0.0);
        assert_ne!(theme.hover_fill(), theme.base.panel);
    }

    #[test]
    fn built_in_theme_presets_have_unique_ids() {
        let mut ids = std::collections::BTreeSet::new();
        for preset in ForgeThemePreset::ALL {
            assert!(ids.insert(ForgeTheme::from_preset(preset).id));
        }
        assert_eq!(ids.len(), ForgeThemePreset::ALL.len());
    }
}
