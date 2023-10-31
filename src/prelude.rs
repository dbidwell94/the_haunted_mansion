pub use crate::utils::*;
use bevy::{
    ecs::query::{ReadOnlyWorldQuery, WorldQuery},
    prelude::*,
};

pub trait GetEitherMut<'world, Element, Filter = ()>
where
    Element: WorldQuery,
    Filter: ReadOnlyWorldQuery,
{
    fn get_either_mut(&mut self, this: Entity, otherwise: Entity) -> Option<Element::Item<'_>>;
}

impl<'world, 'state, Element, Filter> GetEitherMut<'world, Element, Filter>
    for Query<'world, 'state, Element, Filter>
where
    Element: WorldQuery,
    Filter: ReadOnlyWorldQuery,
{
    fn get_either_mut(&mut self, this: Entity, otherwise: Entity) -> Option<Element::Item<'_>> {
        let to_query: Entity;
        if self.get(this).is_ok() {
            to_query = this;
        } else if self.get(otherwise).is_ok() {
            to_query = otherwise;
        } else {
            return None;
        };

        self.get_mut(to_query).ok()
    }
}

pub trait GetEither<'world, Element, Filter = ()>
where
    Element: ReadOnlyWorldQuery,
{
    fn get_either(&self, this: Entity, otherwise: Entity) -> Option<Element::Item<'_>>;
    fn get_either_returning_other(
        &self,
        this: Entity,
        otherwise: Entity,
    ) -> Option<(Element::Item<'_>, Entity)>;
}

impl<'world, 'state, Element, Filter> GetEither<'world, Element, Filter>
    for Query<'world, 'state, Element, Filter>
where
    Element: ReadOnlyWorldQuery,
    Filter: ReadOnlyWorldQuery,
{
    fn get_either(&self, this: Entity, otherwise: Entity) -> Option<Element::Item<'_>> {
        let to_query: Entity;
        if self.get(this).is_ok() {
            to_query = this;
        } else if self.get(otherwise).is_ok() {
            to_query = otherwise;
        } else {
            return None;
        };

        self.get(to_query).ok()
    }

    fn get_either_returning_other(
        &self,
        this: Entity,
        otherwise: Entity,
    ) -> Option<(Element::Item<'_>, Entity)> {
        let to_query: Entity;
        let other: Entity;
        if self.get(this).is_ok() {
            to_query = this;
            other = otherwise;
        } else if self.get(otherwise).is_ok() {
            to_query = otherwise;
            other = this;
        } else {
            return None;
        };

        self.get(to_query).ok().map(|item| (item, other))
    }
}
