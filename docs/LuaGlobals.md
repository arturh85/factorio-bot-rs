Lua Plan Globals
================

world
-----

Methods 
- world.recipe("inserter") 

plan
-----

Methods 
- plan.groupStart("Mine with Bots")
  - opens a new sync group with given label 
- plan.mine(playerId, {0,0}, "test", 1)
  - mines given entity. automatically adds walk if too far away
- plan.groupEnd() 

rcon
-----

Methods 
- rcon.findNearest(search_center, 500, name, entityName, #bots) 
