use crate::{GameSettings, GameState, SoundAssetHolder};
use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl};
use iyes_loopless::condition::{ConditionSet, IntoConditionalSystem};
use iyes_loopless::prelude::AppLooplessStateExt;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffects>();

        app.add_enter_system(GameState::GameSetupOnce, play_bg_music);
        app.add_audio_channel::<Background>();
        app.add_audio_channel::<Effects>();
        app.add_audio_channel::<ShieldAudio>();

        app.add_system(handle_sound_events.run_on_event::<SoundEffects>());
    }
}

struct ShieldAudio;

struct Effects;

struct Background;

pub(crate) enum SoundEffects {
    //GamePlay
    PlanetDamaged,

    //missile
    MissileLaunched,
    MissileExplosion,
    //sonar
    ScanStarted,
    ScanEnemy,
    EnemySpawnWarning,

    //
    ShieldOn(bool),
    ShieldHit,
    //UI
    NormalButton,
    SmallUpgradeButton,
    UpgradeButton,
    ErrorButton,
}

fn play_bg_music(
    sounds: Res<SoundAssetHolder>,
    audio: Res<AudioChannel<Background>>,
    game_settings: Res<GameSettings>,
) {
    if !game_settings.is_sound_on {
        return;
    }

    audio.play(sounds.music.clone()).with_volume(0.10).looped();
}

fn handle_sound_events(
    mut sound_event: EventReader<SoundEffects>,
    sounds: Res<SoundAssetHolder>,
    audio: Res<AudioChannel<Effects>>,
    shield_audio: Res<AudioChannel<ShieldAudio>>,
    game_settings: Res<GameSettings>,
) {
    for event in sound_event.iter() {
        if !game_settings.is_sound_on {
            return;
        }
        match event {
            SoundEffects::NormalButton => {
                audio.play(sounds.normal_button.clone()).with_volume(0.3);
            }
            SoundEffects::SmallUpgradeButton => {
                audio.play(sounds.normal_button.clone()).with_volume(0.3);
            }
            SoundEffects::UpgradeButton => {
                audio.play(sounds.super_upgrade.clone()).with_volume(0.3);
            }

            SoundEffects::ErrorButton => {
                audio.play(sounds.error_button.clone()).with_volume(0.5);
            }

            SoundEffects::PlanetDamaged => {
                audio.play(sounds.planet_damage.clone());
            }
            SoundEffects::MissileLaunched => {
                audio.play(sounds.missile_launch.clone());
            }
            SoundEffects::MissileExplosion => {
                audio.play(sounds.missile_explosion.clone()).with_volume(0.5);
            }

            SoundEffects::ScanStarted => {
                audio.play(sounds.scan_launch.clone()).with_volume(0.3);
            }
            SoundEffects::ScanEnemy => {
                audio.play(sounds.scan_ping.clone()).with_volume(0.3);
            }

            SoundEffects::EnemySpawnWarning => {
                audio.play(sounds.enemy_warning.clone()).with_volume(3.0);
            }

            SoundEffects::ShieldOn(bool) => {
                match *bool {
                    true => {
                        shield_audio.play(sounds.shield_on.clone()).with_volume(3.0).looped();
                    }
                    false => {
                        shield_audio.stop();
                    }
                }
            }
            SoundEffects::ShieldHit => {
                audio.play(sounds.shield_hit.clone()).with_volume(3.0);
            }
        }
    }
}
