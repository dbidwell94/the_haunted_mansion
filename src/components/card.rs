use crate::events::GameEvent;
use bevy::prelude::*;

#[derive(Debug, Clone, Reflect)]
pub enum CardType {
    Event(EventCard),
    Omen(OmenCard),
    Item(ItemCard),
}

#[derive(Reflect, Debug, Clone)]
pub enum ItemCard {
    RabbitsFoot,
    MedicalKit,
    Armor,
    Axe,
    PickpocketsGloves,
    DarkDice,
    AngelFeather,
    BloodDagger,
    Revolver,
    AmuletOfTheAges,
    AdrenalineShot,
    SmellingSalts,
    Bell,
    Candle,
    Bottle,
    PuzzleBox,
    SacrificialDagger,
    Dynamite,
    HealingSalve,
    Idol,
    LuckyStone,
}

#[derive(Reflect, Debug, Clone)]
pub enum EventCard {
    TheBeckoning,
    Groundskeeper,
    TheWalls,
    LockedSafe,
    GraveDirt,
    Skeletons,
    TheVoice,
    ClosetDoor,
    Rotten,
    Footsteps,
    Smoke,
    SecretPassage,
    Whoops,
    MysticSlide,
    MistsFromTheWalls,
    Spider,
    JonahsTurn,
    Silence,
    HangedMen,
    Debris,
    Funeral,
    SecretStairs,
    WhatThe,
    AngryBeing,
    AMomentOfHope,
    Webs,
    DisquietingSounds,
    HideousShriek,
    RevolvingWall,
    CreepyCrawlies,
    BurningMan,
    TheLostOne,
    SomethingHidden,
    BloodyVision,
    CreepyPuppet,
    ImageInTheMirror1,
    Possession,
    ShriekingWind,
    PhoneCall,
    LightsOut,
    DripDripDrip,
    ItIsMeantToBe,
    SomethingSlimy,
    ImageInTheMirror2,
    NightView,
}

#[derive(Reflect, Debug, Clone)]
pub enum OmenCard {
    Girl,
    SpiritBoard,
    Dog,
    Book,
    Madman,
    Medallion,
    HolySymbol,
    Ring,
    Skull,
    CrystalBall,
    Bite,
    Mask,
    Spear,
}

impl GameCard for OmenCard {
    fn process_event(_: GameEvent, _: &mut Commands, _: &Res<AssetServer>) {
        todo!()
    }
}

trait GameCard {
    fn process_event(event: GameEvent, commands: &mut Commands, asset_server: &Res<AssetServer>);
}
