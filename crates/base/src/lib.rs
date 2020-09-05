//! Core functionality for Feather. This crate primarily
//! implements or reexports essential data structures, such as:
//! * Inventories
//! * The block ID system
//! * The chunk data structure
//! * The chunk map (`World`)
//!
//! This crate also exposes the `Setup` and `Tick` types which are
//! used throughout the rest of the codebase.

use ecs::{Ecs, Stage, SysResult, SystemExecutor};
use resources::{CantGetResource, Resource, Resources};

mod chunk;
mod positions;
mod world;

pub use blocks::*;
pub use chunk::{Chunk, ChunkSection, CHUNK_HEIGHT, CHUNK_WIDTH};
pub use generated::{Biome, EntityKind, Item, ItemStack, Particle};
pub use positions::*;
pub use world::World;

/// Struct passed to all systems as their sole argument.
///
/// This type encapsulates all state needed during execution:
/// * The `Ecs`, which stores all entities in the current game.
/// * The `World`, which stores block data for the current world.
/// (This is _not_ the same as the "world" referred to in many Rust
/// ECS libraries; for entities see `Ecs`. We've renamed `World` to `Ecs`
/// to avoid confusion.)
/// * The `Resources`, which store additional unique data that can be used
/// throughout the codebase.
///
/// This struct can be created through `Setup::build()`.
pub struct Tick {
    /// Stores all entities in the current game.
    pub ecs: Ecs,

    resources: Resources,
}

impl Tick {
    /// Gets a reference to the resource with the given type.
    ///
    /// Returns an error if the resource does not exist
    /// (was not inserted with `Setup::resource()`),
    /// or if there is already a mutable borrow to that resource.
    pub fn resource<T: Resource>(&self) -> Result<resources::Ref<T>, CantGetResource> {
        self.resources.get()
    }

    /// Gets a mutable reference to the resource with the given type.
    ///
    /// Returns an error if the resource does not exist
    /// or if there is already a borrow to that resource.
    pub fn resource_mut<T: Resource>(&self) -> Result<resources::RefMut<T>, CantGetResource> {
        self.resources.get_mut()
    }
}

/// Struct passed into functions which set up the server
/// state during startup.
///
/// Currently, this struct is used to:
/// * Register systems with the system executor.
/// * Insert _resources_, data that can be accessed
/// through the method `Tick::resource()`.
#[derive(Default)]
pub struct Setup {
    executor: SystemExecutor<Tick>,
    resources: Resources,
}

impl Setup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a system in the default `Tick` stage.
    ///
    /// The system will be invoked each tick.
    pub fn system(&mut self, system: fn(&mut Tick) -> SysResult) -> &mut Self {
        self.system_in_stage(system, Stage::Tick)
    }

    /// Registers a system to the specified stage.
    ///
    /// The system will be invoked each tick.
    pub fn system_in_stage(
        &mut self,
        system: fn(&mut Tick) -> SysResult,
        stage: Stage,
    ) -> &mut Self {
        self.executor.add_system(stage, system);
        self
    }

    /// Inserts a resource. Systems can then access
    /// this resource by calling `Tick::resource()`.
    ///
    /// Resources can be any type that is `Send + Sync + 'static`.
    pub fn resource(&mut self, resource: impl Resource) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    /// Completes setup, returning a `Tick` and a `SystemExecutor`.
    pub fn build(self) -> (Tick, SystemExecutor<Tick>) {
        (
            Tick {
                resources: self.resources,
                ecs: Ecs::new(),
            },
            self.executor,
        )
    }
}
