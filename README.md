# rustirc

A re-implementation of a vintage IRC bot for fun, learning, and exploration. Some parts are overkill or silly,
while others are underdeveloped. It's probably more useful as an example than a reference.


## capabilities

A partial list of capabilities:

- Rumors like "rustybot, eat your peas" -> `Good to know!` -> "rustybot, peas?" -> `All I know is eat your peas`.
- Track part/join/quit for to store when a user was last seen
- A scoring system. "wonderzombie++" -> `wonderzombie's score is now 2!`


**irc_core** - My extremely spare implementation of the IRC protocol
**bot** - Uses `irc_core` to implement chat-layer functionality
**SQLite schema** is in `bot/src/schema.sql`.
