use crate::{
    archetype::Archetype,
    change_detection::Tick,
    component::{ComponentId, Components},
    entity::Entity,
    query::{
        FilteredAccess, FilteredAccessSet, IterQueryData, NestedQuery, QueryData, QueryFilter,
        ReadOnlyQueryData, With, WorldQuery,
    },
    relationship::Relationship,
    storage::{Table, TableRow},
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};
use core::iter;

pub struct Related<R: Relationship, D: QueryData + 'static, F: QueryFilter + 'static = ()>(
    RelatedInner<R, D, F>,
);

type RelatedInner<R, D, F> = (
    &'static R,
    NestedQuery<D, (F, With<<R as Relationship>::RelationshipTarget>)>,
);

unsafe impl<R: Relationship, D: QueryData + 'static, F: QueryFilter + 'static> WorldQuery
    for Related<R, D, F>
{
    type Fetch<'w> = <RelatedInner<R, D, F> as WorldQuery>::Fetch<'w>;
    type State = <RelatedInner<R, D, F> as WorldQuery>::State;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        <RelatedInner<R, D, F> as WorldQuery>::shrink_fetch(fetch)
    }

    unsafe fn init_fetch<'w, 's>(
        world: UnsafeWorldCell<'w>,
        state: &'s Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        unsafe {
            <RelatedInner<R, D, F> as WorldQuery>::init_fetch(world, state, last_run, this_run)
        }
    }

    const IS_DENSE: bool = <RelatedInner<R, D, F> as WorldQuery>::IS_DENSE;

    unsafe fn set_archetype<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe {
            <RelatedInner<R, D, F> as WorldQuery>::set_archetype(fetch, state, archetype, table)
        };
    }

    unsafe fn set_table<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        table: &'w Table,
    ) {
        unsafe { <RelatedInner<R, D, F> as WorldQuery>::set_table(fetch, state, table) };
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
        <RelatedInner<R, D, F> as WorldQuery>::update_component_access(state, access);
    }

    fn init_nested_access(
        state: &Self::State,
        system_name: Option<&str>,
        component_access_set: &mut FilteredAccessSet,
        world: UnsafeWorldCell,
    ) {
        <RelatedInner<R, D, F> as WorldQuery>::init_nested_access(
            state,
            system_name,
            component_access_set,
            world,
        );
    }

    fn init_state(world: &mut World) -> Self::State {
        <RelatedInner<R, D, F> as WorldQuery>::init_state(world)
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        <RelatedInner<R, D, F> as WorldQuery>::get_state(components)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        <RelatedInner<R, D, F> as WorldQuery>::matches_component_set(state, set_contains_id)
    }

    fn update_archetypes(state: &mut Self::State, world: UnsafeWorldCell) {
        <RelatedInner<R, D, F> as WorldQuery>::update_archetypes(state, world);
    }
}

unsafe impl<R: Relationship, D: QueryData + 'static, F: QueryFilter + 'static> QueryData
    for Related<R, D, F>
{
    const IS_READ_ONLY: bool = D::IS_READ_ONLY;
    type ReadOnly = Related<R, D::ReadOnly, F>;
    type Item<'w, 's> = D::Item<'w, 's>;

    const IS_ARCHETYPAL: bool = false;

    fn shrink<'wlong: 'wshort, 'wshort, 's>(
        item: Self::Item<'wlong, 's>,
    ) -> Self::Item<'wshort, 's> {
        D::shrink(item)
    }

    fn iter_access(_state: &Self::State) -> impl Iterator<Item = crate::query::EcsAccessType<'_>> {
        iter::empty()
    }

    unsafe fn fetch<'w, 's>(
        state: &'s Self::State,
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Option<Self::Item<'w, 's>> {
        unsafe { <RelatedInner<R, D, F> as QueryData>::fetch(state, fetch, entity, table_row) }
            .and_then(|(relationship, query)| query.get_inner(relationship.get()).ok())
    }
}

unsafe impl<R: Relationship, D: ReadOnlyQueryData + 'static, F: QueryFilter + 'static>
    ReadOnlyQueryData for Related<R, D, F>
{
}

// Note that we require `D: ReadOnlyQueryData` for `Related: IterQueryData`
unsafe impl<R: Relationship, D: ReadOnlyQueryData + 'static, F: QueryFilter + 'static> IterQueryData
    for Related<R, D, F>
{
}

#[cfg(test)]
mod tests {
    use bevy_ecs_macros::Component;

    use crate::{
        entity::Entity,
        hierarchy::ChildOf,
        query::Without,
        relationship::Related,
        schedule::{IntoScheduleConfigs, Schedule},
        system::{Query, ScheduleSystem},
        world::World,
    };

    fn run_system<Marker, S: IntoScheduleConfigs<ScheduleSystem, Marker>>(
        world: &mut World,
        system: S,
    ) {
        let mut schedule = Schedule::default();
        schedule.add_systems(system);
        schedule.run(world);
    }

    #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
    struct TestComp(usize);

    #[test]
    fn entity() {
        #[derive(Component, Clone, Copy, PartialEq, Eq)]
        struct TestComp(usize);

        let mut world = World::new();

        let a = world.spawn(TestComp(0)).id();
        let b = world.spawn((TestComp(1), ChildOf(a))).id();

        let sys = move |query: Query<(Entity, Related<ChildOf, Entity>)>| {
            for (child, parent) in &query {
                assert_eq!(child, b);
                assert_eq!(parent, a);
            }
        };

        run_system(&mut world, sys);
    }

    #[test]
    fn shared() {
        let mut world = World::new();

        let [a, b] = core::array::from_fn(|i| world.spawn(TestComp(i)).id());
        world.entity_mut(a).add_child(b);

        let sys = move |query: Query<(&TestComp, Related<ChildOf, &TestComp>)>| {
            for (child_b, parent_a) in &query {
                assert_eq!(parent_a.0, 0);
                assert_eq!(child_b.0, 1);
            }
        };

        run_system(&mut world, sys);
    }

    #[test]
    fn mutable() {
        let mut world = World::new();

        let [a, b] = core::array::from_fn(|i| world.spawn(TestComp(i)).id());
        world.entity_mut(a).add_child(b);

        let sys = move |mut query: Query<(
            &TestComp,
            Related<ChildOf, &mut TestComp, Without<ChildOf>>,
        )>| {
            let mut iter = query.iter_mut();
            while let Some((child_b, mut parent_a)) = iter.fetch_next() {
                *parent_a = *child_b;
                assert_eq!(*parent_a, *child_b);
            }
        };

        run_system(&mut world, sys);
    }
}
