This is a simulation game where actors travel from one place to another
At the start of the game there are few entities spawned on the 10x10 grid
- home (at 1,1 position)
- work (at 6,8 position)
- 3 actors (at random positions)

The game should tick at 2 frames (ticks) per second rate
With every tick the actor can make a move by one unit horizontally, vertically or diagonally
When position of the actor is adjacent (horizontally, diagonally or vertically) to the location (home or work)
the actor should stop for 10 ticks and then travel to the next location - agents should continuously travel 
between home and work in this manner

the game should be displayed in cmd - home indicated by H, work indicated by W and actors inticated by A

Actors and locations (home and work) should be treated as obstacles for the actor navigation and movement system
they should calculate the route to avoid obstacles but in case of error in the route calculation the movement system
should prevent actors from moving into each other or into locations