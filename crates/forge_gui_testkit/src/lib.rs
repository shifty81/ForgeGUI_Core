//! ForgeGUI contract certification helpers.
#![forbid(unsafe_code)]
use forge_gui_core::{
    GuiContributionManifest, InterfacePreset, PanelCatalog, PanelDockPolicy, PanelId,
};
#[derive(Default, Debug)]
pub struct AuditReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
impl AuditReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn assert_ok(&self) {
        assert!(
            self.is_ok(),
            "ForgeGUI contract audit failed: {}",
            self.errors.join(" | ")
        );
    }
}
pub fn audit_manifest(manifest: &GuiContributionManifest) -> AuditReport {
    let mut r = AuditReport::default();
    if let Err(e) = manifest.validate() {
        r.errors.push(e.to_string());
    }
    r
}
pub fn audit_preset(preset: &InterfacePreset, panels: &PanelCatalog) -> AuditReport {
    let mut r = AuditReport::default();
    if let Err(e) = preset.validate(panels) {
        r.errors.push(e.to_string());
    }
    r
}
pub fn audit_catalog(catalog: &PanelCatalog, presets: &[InterfacePreset]) -> AuditReport {
    let mut r = AuditReport::default();
    if catalog.is_empty() {
        r.warnings.push("panel catalog is empty".into());
    }
    for p in presets {
        if let Err(e) = p.validate(catalog) {
            r.errors.push(e.to_string());
        }
    }
    r
}
pub fn audit_dock_policies(policies: &[PanelDockPolicy], panels: &PanelCatalog) -> AuditReport {
    let mut r = AuditReport::default();
    for p in policies {
        if panels.get(&p.panel).is_none() {
            r.errors
                .push(format!("dock policy references missing panel {}", p.panel));
        }
    }
    r
}
pub fn panel_ids(catalog: &PanelCatalog) -> Vec<PanelId> {
    catalog.iter().map(|p| p.id.clone()).collect()
}
