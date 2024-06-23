# Marching cubes based sculpting app

### Technologies

- Rust
- OpenGL

#### Description

Project is based on [Marching Cubes](https://en.wikipedia.org/wiki/Marching_cubes) algorithm. All computation (except raycasting) is done on GPU inside Compute Shaders.

<img src="https://github.com/hevezolly/blobs/blob/master/showcase1.gif" width="600" height="600" />

#### Shading

Shading is based on [Gooch shading](https://en.wikipedia.org/wiki/Gooch_shading) with added Ambient Occlusion term. <br> Ambient occlusion calculation is based on SDF ambient occlusion with few tweaks. Hemisphere sampling direction is mixed with small portion of procedural blue noise. This is how pure ambient occlusion looks:

  <img src="https://github.com/hevezolly/blobs/blob/master/ao.gif" width="300" height="300" />

#### Optimizations

- Scalar field update happens only in "dirty" area, around user interraction point.
- Mesh recalculation using Marching Cubes also happens in that area. To achive that, two shader dispatches performed: fist one runs over dirty area and updates vertex information in sparse Vertex buffer. After that another shader runs over whole sdf field and builds Index buffer using atomics.
- To perform raycast check results of Marching Cubes work have to be sent from gpu to cpu. To decrease size of transfered memory collision shape is simplified. Only 1 byte required for every cube in a field. By packing - only 1 uint is required for every 4 cubes. This is how the collider shape looks:

  <img src="https://github.com/hevezolly/blobs/blob/master/collider.gif" width="300" height="300" />

Thus, field of size 64x64x64 requires only 6-7ms to update on my machine.



