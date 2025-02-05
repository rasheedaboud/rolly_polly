use bevy::prelude::*;
use bevy_audio_controller::prelude::{AudioFiles, GlobalPlayEvent};

// SYSTEMS -------------------------------------

fn setup_audio(mut sfx_play_ew: EventWriter<GlobalPlayEvent>) {
    let event = GlobalPlayEvent::new(AudioFiles::Song18MP3).with_settings(PlaybackSettings::LOOP);
    sfx_play_ew.send(event);
}

// PLUGIN -------------------------------------

pub struct BackgroundAudioPlugin;

impl Plugin for BackgroundAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio);
    }
}
