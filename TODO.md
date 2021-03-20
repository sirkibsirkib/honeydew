1. define a replacement for Vec2, for a modulo N coordinate space, with functionality:
	- invariant: values are in range 0..N where N is some constant
	- modulo N add / sum to points
	- modulo N negation
	- abs() returns OrderedFloat, representing the distance from 0. EITHER WAY AROUND

1. rebuild Pos type using discrete integer storage using the MSB.
	modulo arithmetic using all MSB is super cheap! Just allow operations to overflow
