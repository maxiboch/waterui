//! Baseline probe for the WaterUI upgrade.
//!
//! This exists to answer one question before any porting work starts: does
//! contemporary upstream WaterUI build and render for the shapes our own
//! consumers need? It deliberately measures UPSTREAM, not a port, so it uses
//! nothing but the public API and the upstream test harness.
//!
//! What it covers, and why each piece is here:
//!
//! - A view built from `prelude` primitives, which is the whole of what
//!   voicemaci asks of WaterUI (`App`, `prelude`, `AnyView`, `media::Image`).
//! - An `app()` entry point, so the same view can be run on a desktop backend
//!   by hand rather than only under test.
//! - A headless render through `hydrolysis_m3`, which is the path maxi-tray,
//!   maxi-visage and the gallery use, and the only one available on a fleet
//!   whose Macs are screen-locked.
//!
//! It is intentionally not a HUD. A GPU surface and an overlay window are the
//! next probe; this one establishes that the workspace resolves and the
//! headless lane renders at all.

use waterui::app::App;
use waterui::prelude::*;
use waterui::preview;

/// A caption strip, shaped like the HUD's text line without any of its
/// machinery — enough to prove text lowers and lays out headlessly.
fn caption(line: &'static str) -> impl View {
    text(line)
}

/// The probe's view: three caption lines, nothing else.
#[preview]
pub fn demo() -> impl View {
    vstack((
        caption("phase0 probe"),
        caption("upstream baseline"),
        caption("headless render"),
    ))
}

/// Desktop entry point, so the same view can be run on a real backend by hand
/// rather than only under the headless harness.
pub fn app(env: Environment) -> App {
    App::new(demo, env)
}

#[cfg(test)]
mod tests {
    use super::demo;
    use waterui_testing::UiBuilder;

    /// The probe renders headlessly and its content is reachable by query.
    ///
    /// A green result here means the workspace resolves, the theme installs,
    /// and the hydrolysis lane produces a tree we can assert against — the
    /// three things every later phase depends on.
    #[waterui::test(theme = hydrolysis_m3::install, viewport = (480, 240))]
    fn probe_renders_headless(ui: UiBuilder) {
        let mut app = ui.mount(demo);

        for line in ["phase0 probe", "upstream baseline", "headless render"] {
            app.query().label(line).assert_exists();
        }
    }
}
