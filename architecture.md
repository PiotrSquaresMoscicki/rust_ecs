Entity - unique ID
Component - data assigned to entity
System - logic operating on components
World - collection of entities, components, systems and child worlds

Code and pseudocode samples

```
// system trait
pub trait System {
    // Dependencies is a tuple of system types in number from zero to 64
    // Dependencies are other systems that must be updated before this system
    type Dependencies;

    // Input components are components that the system will read from without modifying them
    type InputComponents;
    // Output components are components that the system will read from and write to
    type OutputComponents;

    fn initialize(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World);
    fn denitialize(&mut self, world: &mut World);
}

// since world contains systems of different types we need a SystemWrapper that will
// get required iterators and call the system's update method
struct SystemWrapper<S: System> {
    system: S,
}

impl<S: System> SystemWrapper<S> {
    fn update(&mut self, world: &mut World) -> SystemUpdateDiff {
        // create a snapshot of system output components
        // ...

        // create a snapshot of the system state
        // ...

        // call update
        self.system.update(world);

        // diff the snapshot with current state and record changes
        // ...

        SystemUpdateDiff { /* ... */ }
    }
}

// sample system implementation
struct SampleSystem;
impl System for SampleSystem {
    type Dependencies = (OtherSystem,);
    type Iterators = (
        // EntIt is a marker type for clarity of reading - to make it easier to count how many iterators we're using 
        // and what components are used by each iterator 
        EntIt<Mut<ComponentA>, ComponentB>,
        EntIt<ComponentC>,
    );

    fn update(&mut self, &mut iterators: Self::Iterators) {
        let (mut iter_a_b, iter_c) = iterators;

        while let Some((entity, &mut comp_a, &comp_b)) = iter_a_b.next() {
            // process entity with ComponentA and ComponentB
            // comp_a is mutable
            // comp_b is immutable
        }

        while let Some((&entity, &comp_c)) = iter_c.next() {
            // process entity with ComponentC
            // comp_c is immutable
        }
    }

    fn on_component
}

// An Entity is just a unique identifier.
pub struct Entity(usize);

// world pseudocode
struct World {
    entities: Vec<Entity>,
    components: HashMap<TypeId, Vec<Component>>,
    systems: Vec<Box<dyn Any>>,
    next_entity_id: usize,
    world_update_history: WorldUpdateHistory,
}

impl World {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
            systems: Vec::new(),
            next_entity_id: 0,
            world_update_history: WorldUpdateHistory::new(),
        }
    }

    fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(SystemWrapper { system }));
    }

    fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(Vec::new)
            .push((entity, Box::new(component)));
    }

    fn initialize_systems(&mut self) {
        for system in &mut self.systems {
            if let Some(wrapper) = system.downcast_mut::<SystemWrapper<impl System>>() {
                // Call any initialization logic here if needed
            }
        }
    }

    fn update(&mut self) {
        // create object for tracking changes in this update
        let mut world_update_diff = WorldUpdateDiff::new();

        for system in &mut self.systems {
            let system_diff = system.update(self);
            world_update_diff.record(system_diff);
        }

        self.world_update_history.record(world_update_diff);
    }

    fn get_iterators<I>(&self) -> I {
        // Logic to construct and return the required iterators based on I
        unimplemented!()
    }
}

// execution overview
fn main() {
    // create world
    let mut world = World::new();

    // register systems
    world.add_system::<SampleSystem>();

    // initialize systems - one time function call before the first update so systems can setup their state
    world.initialize_systems();

    loop {
        world.update();
    }
}
```