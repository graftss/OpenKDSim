# OpenKDSim

This project is an open-source reimplementation of the `PS2KatamariSimulation.dll` library of Katamari Damacy Reroll. It contains the game's root "tick" routine, as well as all physics and collision code (both for the katamari and for collectible objects) that underlies the core gameplay of Katamari Damacy. The game essentially treats this DLL as a black box "backend" (as opposted to the Unity-based "frontend") in that it (1) calls the tick routine, (2) requests the position and orientation of the katamari and of every game object, and (3) repositions the katamari and game objects to match that data (in the Unity frontend). 

The original DLL's API is reimplemented in `src/lib.rs`.
