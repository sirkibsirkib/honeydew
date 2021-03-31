# Honeydew
A simple multiplayer game, in which three players chase each other, rock-paper-scissors style. 
Named in honor of the place that inspired it: the [Honeydew Mazes](http://honeydewmazes.co.za) in Johannesburg, South Africa,
where my dad took me and my two friends (Jaco & Anthony), when we were kids.

## Gameplay
Three players, colored Black, Blue, and Orange, come together in a game. The game world is a randomly generated grid maze of horizontal and vertical walls.
Players can move in 2-dimensional space using the WASD keys. As a player, you are concerned with colliding with one of the other entities in the game:
1. your prey: colliding with the player who is your prey "kills" them, relocating them randomly in the maze. You want this.
2. your predator: colliding with the player whose prey you are "kills" you, relocates you randomly in your maze. You do not want this.
3. teleporters: upon collision, you and the teleporter itself are randomly relocated in the maze.
4. walls: brown walls divide the playspace into a maze. They do nothing but impede your movement.
4. doors: walls that are lightly colored allow you move through them like doors. After being touched, they relocate to a random wall in the room. Note that doors are personal to the player. Each player can only see or use their own doors. You never know exactly where your peers can move!