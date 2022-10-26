use crate::{GameSettings, GameState, SoundAssetHolder};
use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl};
use iyes_loopless::condition::{ConditionSet, IntoConditionalSystem};
use iyes_loopless::prelude::AppLooplessStateExt;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffectEvents>();
        app.add_event::<SoundSettingsEvents>();

        app.add_enter_system(GameState::GameSetupOnce, play_bg_music);
        app.add_audio_channel::<Background>();
        app.add_audio_channel::<Effects>();
        app.add_audio_channel::<ShieldAudio>();

        app.add_system(handle_sound_events.run_on_event::<SoundEffectEvents>());
        app.add_system(handle_sound_settings.run_on_event::<SoundSettingsEvents>());
    }
}

struct ShieldAudio;

struct Effects;

struct Background;

pub(crate) enum SoundSettingsEvents {
    SoundToggle(bool),
    BGToggle(bool),
    SoundVolumeMaster(f64),
    SoundVolumeBg(f64),
    SoundVolumeEffects(f64),
}

pub(crate) enum SoundEffectEvents {
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

fn handle_sound_settings(
    sounds: Res<SoundAssetHolder>,
    bg_audio: Res<AudioChannel<Background>>,
    effect_audio: Res<AudioChannel<Effects>>,
    shield_audio: Res<AudioChannel<ShieldAudio>>,
    game_settings: Res<GameSettings>,
    mut events: EventReader<SoundSettingsEvents>,
) {
    for event in events.iter() {
        match event {
            SoundSettingsEvents::SoundToggle(bool) => {
                if *bool == true && game_settings.is_bg_sound_on {
                    bg_audio
                        .play(sounds.music.clone())
                        .with_volume(0.10)
                        .looped();
                } else {
                    bg_audio.stop();
                }
            }
            SoundSettingsEvents::BGToggle(bool) => {
                if *bool == true && game_settings.is_sound_on {
                    bg_audio
                        .play(sounds.music.clone())
                        .with_volume(0.10)
                        .looped();
                } else {
                    bg_audio.stop();
                }
            }

            SoundSettingsEvents::SoundVolumeMaster(volume) => {
                bg_audio.set_volume(*volume * game_settings.bg_sound_level.1 as f64);
                effect_audio.set_volume(*volume * game_settings.effects_sound_level.1 as f64);
                shield_audio.set_volume(*volume * game_settings.effects_sound_level.1 as f64);
            }
            SoundSettingsEvents::SoundVolumeBg(_) => {}
            SoundSettingsEvents::SoundVolumeEffects(_) => {}
        }
    }
}

fn play_bg_music(
    mut sound_settings_writer: EventWriter<SoundSettingsEvents>,
    game_settings: Res<GameSettings>,
) {
    sound_settings_writer.send(SoundSettingsEvents::SoundToggle(game_settings.is_sound_on));
}

fn handle_sound_events(
    mut sound_event: EventReader<SoundEffectEvents>,
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
            SoundEffectEvents::NormalButton => {
                audio
                    .play(sounds.normal_button.clone())
                    .with_volume(0.3 * game_settings.effects_sound_level.1);
            }
            SoundEffectEvents::SmallUpgradeButton => {
                audio
                    .play(sounds.normal_button.clone())
                    .with_volume(0.3 * game_settings.effects_sound_level.1);
            }
            SoundEffectEvents::UpgradeButton => {
                audio
                    .play(sounds.super_upgrade.clone())
                    .with_volume(0.3 * game_settings.effects_sound_level.1);
            }

            SoundEffectEvents::ErrorButton => {
                audio
                    .play(sounds.error_button.clone())
                    .with_volume(0.5 * game_settings.effects_sound_level.1);
            }

            SoundEffectEvents::PlanetDamaged => {
                audio.play(sounds.planet_damage.clone());
            }
            SoundEffectEvents::MissileLaunched => {
                audio.play(sounds.missile_launch.clone());
            }
            SoundEffectEvents::MissileExplosion => {
                audio
                    .play(sounds.missile_explosion.clone())
                    .with_volume(0.5 * game_settings.effects_sound_level.1);
            }

            SoundEffectEvents::ScanStarted => {
                audio
                    .play(sounds.scan_launch.clone())
                    .with_volume(0.3 * game_settings.effects_sound_level.1);
            }
            SoundEffectEvents::ScanEnemy => {
                audio
                    .play(sounds.scan_ping.clone())
                    .with_volume(0.3 * game_settings.effects_sound_level.1);
            }

            SoundEffectEvents::EnemySpawnWarning => {
                audio
                    .play(sounds.enemy_warning.clone())
                    .with_volume(2.0 * game_settings.effects_sound_level.1);
            }

            SoundEffectEvents::ShieldOn(bool) => match *bool {
                true => {
                    shield_audio
                        .play(sounds.shield_on.clone())
                        .with_volume(3.0 * game_settings.effects_sound_level.1)
                        .looped();
                }
                false => {
                    shield_audio.stop();
                }
            },
            SoundEffectEvents::ShieldHit => {
                audio
                    .play(sounds.shield_hit.clone())
                    .with_volume(3.0 * game_settings.effects_sound_level.1);
            }
        }
    }
}
