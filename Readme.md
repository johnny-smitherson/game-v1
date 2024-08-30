# TODO - gameplay feat

- [x] tank controlller
- [x] tank shooting
- [x] bunch of tanks
- [x] minimap UI camera
- [x] bullet kills tanks
- [ ] multiple proposed trajectories
- [ ] proposed trajectories check terrain
- [ ] death explosion effect
- [ ] player tank moves to right click
- [ ] power/elevation buttons keep same target
- [ ] flight time plus/minus keep same target
- [x] AI contorolled tank - shoot closest, move randomly
- [ ] multiplayer https://johanhelsing.studio/posts/extreme-bevy

# TODO - gameplay feat ideas
- each player controls 1 artillery tank + 1 drone
- line of sight fog of war (based on all team drones + tank line of sight)
- player sees enemy shell impact point from radar when close to impact (and extrapolates shooter position) - needs drone for accurate target info
- different types of ammo in each gun class (killstreaks? cs-style money points?)
- different types of gun classes (like battlefield)
- game mode:
  - sp PVE
  - mp FFA teams 1-4 (fortnite)
  - mp Red Vs. Blue (2-3 fronts)
- achievements pve
- elo rating pvp
- fortnite style powerup on map
- skill based match making on same map, spatially (east easier, west harder)


# TODO - graphics feat

- [ ] parallax mapping height map triangles
  - [ ] split tiles each 1 entity
  - [ ] material texture generation
- [ ] post processing
  - [ ] post-proc lines shader
  - [ ] depth of field https://www.youtube.com/watch?v=v9x_50czf-4
  - [ ] grass https://www.youtube.com/watch?v=bp7REZBV4P4
- [ ] volumetric? clouds & smoke

# TODO - non-trivial
- [ ] app-world compute shaders? https://github.com/Kjolnyr/bevy_app_compute/pull/4
- [ ] upgrade bevy 0.12
  - audio system rewrite - 
  - bevy-hanabi update
  - physics libs update
  - 


# TODO - WASM WEBGL INCOMPATIBLE CRATES

- "hanabi" - particle effects (compute shaders)
   - unsupported webgpu https://github.com/djeedai/bevy_hanabi/issues/41
   - some fork might work, but disable world inspector plugin 
- "bevy_atmosphere" - Nishta sky (compute shadefrs)
   - unsupported webgpu
- "simdnoise" - only works intel SIMD


# TODO - genetic AI
- https://www.youtube.com/watch?v=N3tRFayqVtk
