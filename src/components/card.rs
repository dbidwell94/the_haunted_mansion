use crate::events::GameEvent;
use bevy::prelude::*;

#[derive(Reflect, Debug, Clone)]
pub enum CardType {
    Event(EventCard),
    Omen(OmenCard),
    Item(ItemCard),
}

#[derive(Reflect, Debug, Clone)]
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

#[derive(Reflect, Debug, Clone)]
pub enum EventCard {}

#[derive(Reflect, Debug, Clone)]
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
    fn process_event(_: GameEvent) {
        todo!()
    }
}

trait GameCard {
    fn process_event(event: GameEvent);
}
