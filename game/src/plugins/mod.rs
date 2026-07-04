pub mod audio;
pub mod debug;
pub mod feedback;
pub mod feel_test;
pub mod input;
pub mod render;
pub mod ui;
pub mod vfx;

use std::fmt;

use feedback_events::AccessibilityFeedbackFlags;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginRegistrationSummary {
    pub ui: ui::UiRegistrationSummary,
    pub audio: audio::AudioRegistrationSummary,
    pub vfx: vfx::VfxRegistrationSummary,
    pub debug: debug::DebugRegistrationSummary,
}

impl fmt::Display for PluginRegistrationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "plugins registered: ui(first_bounce={}, balls={}, equipped_skills={}, power={}%), audio(cues={}, high_freq={}), vfx(events={}, cues={}, shake={}), debug(collisions={}, first_bounce={}, reused_aim={})",
            self.ui.first_bounce_predicted,
            self.ui.balls_remaining,
            self.ui.equipped_skill_count,
            self.ui.active_power_charge_percent,
            self.audio.cues,
            self.audio.high_frequency_cues,
            self.vfx.events,
            self.vfx.cues,
            self.vfx.camera_shake_cues,
            self.debug.collision_events,
            self.debug.first_bounce_predicted,
            self.debug.reused_aim_bounce_predicted,
        )
    }
}

pub fn register_placeholders() -> PluginRegistrationSummary {
    render::register();
    input::register();
    let _feedback_cue_count = feel_test::run_smoke_scene()
        .ok()
        .map(|scene| {
            feedback::play_feel_test_shot(
                scene.seed,
                &scene.board,
                &scene.input,
                AccessibilityFeedbackFlags::DEFAULT,
            )
            .summaries
            .len()
        })
        .unwrap_or(0);

    PluginRegistrationSummary {
        ui: ui::register(),
        audio: audio::register(),
        vfx: vfx::register(),
        debug: debug::register(),
    }
}
