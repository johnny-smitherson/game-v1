# TODO - gameplay feat

- [x] tank controlller
- [x] tank shooting
- [x] bunch of tanks
- [x] minimap UI camera
- [ ] bullet kills tanks
- [ ] death explosion effect
- [ ] player tank moves to right click
- [ ] power/elevation buttons keep same target
- [ ] flight time plus/minus keep same target
- [ ] AI contorolled tank - shoot closest, move randomly
- [ ] multiplayer https://johanhelsing.studio/posts/extreme-bevy
- game mode:
  - sp PVE
  - mp FFA teams 1-4 (fortnite)
  - mp Red Vs. Blue (2-3 fronts)
- achievements pve
- elo rating pvp
- fortnite style powerup pe harta
- skill based match making dar in acelasi univers, asesat spatial


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
