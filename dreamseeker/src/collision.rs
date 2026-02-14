use avian3d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameLayer {
    #[default]
    Level,
    Player,
    Attackable,
    Sword,
    Sensor,
}
