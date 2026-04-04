# vivarium

A realtime 3D simulation exploring multiple interacting behaviors in a dynamic spatial environment, built with [Bevy](https://bevyengine.org/).

## Current systems

- **Insect swarms** — Brownian motion wandering, slow and erratic
- **Predator birds** — loose flocking (boids), cone-of-sight hunting, eat insects on contact

Insects respawn to maintain population balance.

## Running

```
cargo run
```

## Architecture

ECS composition with parallel-friendly systems:

```
rebuild_spatial_index
    |
brownian_motion  ||  flocking    (parallel — disjoint entity sets)
    |                   |
         predator_sight
              |
          movement
              |
      eating + respawn
              |
      boundary_wrap (PostUpdate)
```

3D spatial index (HashMap grid with 27-cell neighborhood queries) enables efficient neighbor lookups for flocking, predator sight, and eating.

## Tests

```
cargo test
```
