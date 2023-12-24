use legion::{systems, Resources, World};

/// Entry point for building a legion schedule
pub trait Scene {
    /// Initialize the world for this scene
    fn init(&mut self, _world: &mut World, _resources: &mut Resources) {}

    /// Schedule systems to run during an update
    fn update(&mut self, sched: &mut systems::Builder);

    /// Schedule systems to run during a draw
    fn draw(&mut self, sched: &mut systems::Builder);
}

// We need to think about how scene transitions should work. World data should probably not be lost
// during a scene transition.
//
// Example cutscene->FPS should not have to rebuild the world.
// However there may is an issue. The component structure of the world affects the code side.
// I still kinda think we shouldn't expose the ECS to the engine user directly.
//
// There should be an ability to change the world when a scene is started.
