use crate::events::GameEvent;
use bevy::prelude::*;

#[derive(Default, Clone, Copy, Hash, PartialEq, Debug, Reflect)]
pub enum CardTypeNoValue {
    Event,
    Omen,
    Item,
    #[default]
    None,
}

pub enum CardType {
    Event(EventCard),
    Omen(OmenCard),
    Item(ItemCard),
}

pub enum ItemCard {
    RabbitsFoot,
    AngelsFeather,
    Dynamite,
    Gun,
    Crossbow,
    Machete,
    LuckyCoin,
    Chainsaw,
    Map,
    MysticalStopwatch,
    SkeletonKey,
    NecklaceOfTeeth,
    LeatherJacket,
    FirstAidKit,
    StrangeMedicine,
    Brooch,
    StrangeAmulet,
    Headphones,
    Flashlight,
    MagicCamera,
    CreepyDoll,
    Mirror,
}

pub enum EventCard {}

pub enum OmenCard {
    Idol,
    Armor,
    Ring,
    Dog,
    Book,
    Dagger,
    HolySymbol,
    Skull,
    Mask,
}

impl GameCard for OmenCard {
    fn process_event(event: GameEvent) {
        todo!()
    }
}

trait GameCard {
    fn process_event(event: GameEvent);
}
