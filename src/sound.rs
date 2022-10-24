use bevy::prelude::*;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffects>();
    }
}

pub(crate) struct PlaySoundEffect(SoundEffects);

pub(crate) enum SoundEffects {
    //GamePlay
    PlanetDamaged,
    //UI
    UpgradeButton,
    NormalButton,
}
