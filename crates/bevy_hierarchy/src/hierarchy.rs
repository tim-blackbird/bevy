use crate::components::Children;
use bevy_ecs::{
    entity::Entity,
    system::{Command, EntityCommands},
    world::{EntityWorldMut, World},
};

/// Despawns the given entity's children recursively
#[derive(Debug)]
pub struct DespawnChildrenRecursive {
    /// Target entity
    pub entity: Entity,
}

impl Command for DespawnChildrenRecursive {
    fn apply(self, world: &mut World) {
        #[cfg(feature = "trace")]
        let _span = bevy_utils::tracing::info_span!(
            "command",
            name = "DespawnChildrenRecursive",
            entity = bevy_utils::tracing::field::debug(self.entity)
        )
        .entered();

        world.entity_mut(self.entity).remove::<Children>();
    }
}

/// Trait that holds functions for despawning recursively down the transform hierarchy
pub trait DespawnRecursiveExt {
    /// Despawns all descendants of the given entity.
    fn despawn_descendants(&mut self) -> &mut Self;
}

impl<'w, 's, 'a> DespawnRecursiveExt for EntityCommands<'w, 's, 'a> {
    fn despawn_descendants(&mut self) -> &mut Self {
        let entity = self.id();
        self.commands().add(DespawnChildrenRecursive { entity });
        self
    }
}

impl<'w> DespawnRecursiveExt for EntityWorldMut<'w> {
    fn despawn_descendants(&mut self) -> &mut Self {
        let entity = self.id();

        #[cfg(feature = "trace")]
        let _span = bevy_utils::tracing::info_span!(
            "despawn_descendants",
            entity = bevy_utils::tracing::field::debug(entity)
        )
        .entered();

        self.world_scope(|world| {
            world.entity_mut(entity).remove::<Children>();
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::{
        component::Component,
        system::{CommandQueue, Commands},
    };

    use super::DespawnRecursiveExt;
    use crate::{child_builder::BuildChildren, components::Children, HierarchyPlugin};

    #[derive(Component, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
    struct Idx(u32);

    #[derive(Component, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
    struct N(String);

    #[test]
    fn despawn() {
        let mut app = App::new();
        app.add_plugins(HierarchyPlugin::default());
        let world = &mut app.world;
        let mut queue = CommandQueue::default();
        let grandparent_entity;
        {
            let mut commands = Commands::new(&mut queue, &world);

            commands
                .spawn((N("Another parent".to_owned()), Idx(0)))
                .with_children(|parent| {
                    parent.spawn((N("Another child".to_owned()), Idx(1)));
                });

            // Create a grandparent entity which will _not_ be deleted
            grandparent_entity = commands.spawn((N("Grandparent".to_owned()), Idx(2))).id();
            commands.entity(grandparent_entity).with_children(|parent| {
                // Add a child to the grandparent (the "parent"), which will get deleted
                parent
                    .spawn((N("Parent, to be deleted".to_owned()), Idx(3)))
                    // All descendants of the "parent" should also be deleted.
                    .with_children(|parent| {
                        parent
                            .spawn((N("First Child, to be deleted".to_owned()), Idx(4)))
                            .with_children(|parent| {
                                // child
                                parent.spawn((
                                    N("First grand child, to be deleted".to_owned()),
                                    Idx(5),
                                ));
                            });
                        parent.spawn((N("Second child, to be deleted".to_owned()), Idx(6)));
                    });
            });

            commands.spawn((N("An innocent bystander".to_owned()), Idx(7)));
        }
        queue.apply(world);

        let parent_entity = world.get::<Children>(grandparent_entity).unwrap()[0];

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent_entity).despawn();
            // despawning the same entity twice should not panic
            commands.entity(parent_entity).despawn();
        }
        queue.apply(world);

        let mut results = world
            .query::<(&N, &Idx)>()
            .iter(&world)
            .map(|(a, b)| (a.clone(), *b))
            .collect::<Vec<_>>();
        results.sort_unstable_by_key(|(_, index)| *index);

        {
            let children = world.get::<Children>(grandparent_entity);
            assert!(
                children.is_none(),
                "grandparent should no longer have a Children component because its last child has been removed"
            );
        }

        assert_eq!(
            results,
            vec![
                (N("Another parent".to_owned()), Idx(0)),
                (N("Another child".to_owned()), Idx(1)),
                (N("Grandparent".to_owned()), Idx(2)),
                (N("An innocent bystander".to_owned()), Idx(7))
            ]
        );
    }

    #[test]
    fn despawn_descendants() {
        let mut app = App::new();
        app.add_plugins(HierarchyPlugin::default());
        let world = &mut app.world;
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let parent = commands.spawn_empty().id();
        let child = commands.spawn_empty().id();

        commands
            .entity(parent)
            .add_child(child)
            .despawn_descendants();

        queue.apply(world);

        // The parent's Children component should be removed.
        assert!(world.entity(parent).get::<Children>().is_none());
        // The child should be despawned.
        assert!(world.get_entity(child).is_none());
    }

    #[test]
    fn spawn_children_after_despawn_descendants() {
        let mut app = App::new();
        app.add_plugins(HierarchyPlugin::default());
        let world = &mut app.world;
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let parent = commands.spawn_empty().id();
        let child = commands.spawn_empty().id();

        commands
            .entity(parent)
            .add_child(child)
            .despawn_descendants()
            .with_children(|parent| {
                parent.spawn_empty();
                parent.spawn_empty();
            });

        queue.apply(world);

        // The parent's Children component should still have two children.
        let children = world.entity(parent).get::<Children>();
        assert!(children.is_some());
        assert!(children.unwrap().len() == 2_usize);
        // The original child should be despawned.
        assert!(world.get_entity(child).is_none());
    }
}
