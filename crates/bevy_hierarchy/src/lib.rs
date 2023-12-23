#![warn(missing_docs)]
//! `bevy_hierarchy` can be used to define hierarchies of entities.
//!
//! Most commonly, these hierarchies are used for inheriting `Transform` values
//! from the [`Parent`] to its [`Children`].

mod components;
pub use components::*;

mod hierarchy;
pub use hierarchy::*;

mod child_builder;
pub use child_builder::*;

mod events;
pub use events::*;

mod valid_parent_check_plugin;
pub use valid_parent_check_plugin::*;

mod query_extension;
pub use query_extension::*;

#[doc(hidden)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{child_builder::*, components::*, hierarchy::*, query_extension::*};

    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::{HierarchyPlugin, ValidParentCheckPlugin};
}

#[cfg(feature = "bevy_app")]
use bevy_app::prelude::*;
use bevy_ecs::{
    component::ComponentId,
    entity::Entity,
    system::{CommandQueue, Despawn},
    world::DeferredWorld,
};

/// The base plugin for handling [`Parent`] and [`Children`] components
#[derive(Default)]
pub struct HierarchyPlugin;

#[cfg(feature = "bevy_app")]
impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Children>()
            .register_type::<Parent>()
            .register_type::<smallvec::SmallVec<[bevy_ecs::entity::Entity; 8]>>()
            .add_event::<HierarchyEvent>();

        app.world
            .register_component::<Children>()
            .on_remove(on_remove_children);
        app.world
            .register_component::<Parent>()
            .on_remove(on_remove_parent);
    }
}

fn on_remove_children(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
    let mut queue = CommandQueue::default();

    let children = world.get::<Children>(entity);
    for &entity in children.into_iter().flatten() {
        queue.push(Despawn { entity });
    }

    world.commands().append(&mut queue);
}

fn on_remove_parent(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
    let &Parent(parent_entity) = world.get::<Parent>(entity).unwrap();
    world.send_event(HierarchyEvent::ChildRemoved {
        child: entity,
        parent: parent_entity,
    });

    let Some(mut parent) = world.get_entity_mut(parent_entity) else {
        return;
    };

    if let Some(mut parent_children) = parent.get_mut::<Children>() {
        parent_children.0.retain(|child| *child != entity);
        if parent_children.is_empty() {
            world.commands().entity(parent_entity).remove::<Children>();
        }
    }
}
