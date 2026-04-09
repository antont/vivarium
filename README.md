# vivarium

A realtime 3D ecosystem simulation with multiple interacting species, built with [Bevy](https://bevyengine.org/).

## Live on the Web
- https://an.org/vivarium/

## What happens here?

Insects swarm through the air with Brownian motion. Birds flock and hunt them with cone-of-sight predation, then nest on tree branches, lay eggs, and raise hatchlings. Squirrels climb the L-system trees along a navigation graph, detect unguarded hatchlings, and stalk them — triggering parent birds to rush back and defend. Wind bends the trees and drifts the insects.

## Running

### Native (Desktop)
```bash
cargo run
```

### Web Development
```bash
trunk serve
```

### Web Build & Deploy
```bash
./build.sh   # trunk build --release + brotli compression
./up.sh       # upload to GCS
```

## Demos

```bash
cargo run --example nesting_demo     # bird lifecycle: hunt, nest, incubate, hatch
cargo run --example predation_demo   # squirrel vs hatchling, parent bird defense
```

## Architecture

ECS composition with logic/visual separation — spawn systems create logic-only entities, reactive visual systems in `VivariumPlugin` auto-dress them with meshes:

```
wind_update
rebuild_spatial_index
    |
brownian_motion  ||  swarm_cohesion  ||  flocking    (parallel)
    |
hunt_system → bird_fly_to_target → movement → eating
    |
bird_lifecycle → hatchling_alert
    |
boundary_force (PostUpdate)

Visual systems (parallel):
  bird_visual, insect_visual, squirrel_visual, nest_visual, hatchling_visual

Squirrel systems (parallel):
  hatchling_detection, behavior, movement, flee
```

3D spatial index (HashMap grid with 27-cell neighborhood queries) enables efficient neighbor lookups. L-system trees provide navigation graphs for squirrel pathfinding.

## Tests

```bash
cargo test
```
