Need to do some thinking about how to structure the engine as a whole.

I want it to be highly plugin based. I want to be able to easily reuse plugins.

which means I need a good way of communicating between plugins.

Some plugins will be considered so basic that they will be included in the engine by default
    - Rendering
    - Scene Managment
    - Physics
    - Audio
    - Camera interface
    - Input mapper

Others may be common in nearly all games but external
    - Fps camera controller
    - Asset loading
    - Scripting languages

they all need to communicate together. For the most part though there should be a dependecy heirarchy for instance a camera controller will be sending commands to the camera interface but the camera interface doesn't need to know the camera controller exists.

However there will also be limited communication upward. For intance the camera controller will need to source events from the input plugin.

We need to be better about understanding who needs to observe what.

There needs to be a good way of subscribing to events. For instance a horror game might want to be able to subscribe to the camera's position so it can jump scare the player by putting things behind the player. I'd also rather not reimplement event bus for everything that might want it.

we could also do something with wrapping the service locator + observer pattern

struct Service {
    concrete: fn(),
    observers: Vec<fn()>,
}

then when the service is called it'll call all observers before calling the concrete implementation

we would then need to be able to register services with a service locator.

let camera = service("camera");
camera.set.subscribe(|args| log!("Camera moved: {args:?}"));

// now when we call camera.set from anywhere our closure gets called
camera.set(pos, rot);

// getting this to work well from a c api may be tricky
