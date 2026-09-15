use forge_gui_universal::prelude::*;
fn main() {
    let policy = AppPolicy::for_profile(AppProfile::Utility);
    assert!(policy.content_first);
    let mut selection = SelectionService::default();
    selection.set(
        "smoke",
        vec!["record.001".into()],
        Some("record.001".into()),
    );
    assert_eq!(selection.len(), 1);
    println!("ForgeGUI contracts smoke: PASS");
}
