# TODO LIST

- DONE: Get list of all players (INVITE DONE & advanced)
- DONE: Get ONLY SCRIM logs, (make sure that the core players are on the same team, around scrim times) after end of season, so data is recent
- DONE: Ensure no duplicates

What would I need to store?

- teams involved
- who won
- link to log
- score
- is scrim or match?

I could just JSON this all to my own PC and process the data on my own
I believe getting all the logs then organizing it in schema -> to db is better + more learning?

Later

- DONE: Put into DB, (find DB to use start learning it)
- Extract insights, predicting who will win by round, etc
- Generate power rankings

Win percentage across maps, adjusted for each opponent

Advanced stats that I want to track:

- Sac efficiency (times that sac killed/forced medic, or killed demoman per attempts)
  Comments from the file taken out of here.
```
// Bomb efficiency
// Starts on RocketJumpLanded.
// If kill/opposing medic force/ then successful within threshold
// If die without any of these conditions (within threshold), unsucessful.

// things to consider: jumps that were just for reposition/spamming
// how do i make sure that they have jumped enough into the teams?

// Jump in
// Deal damage
// (potentially force/get kill/etc)
// die (or live?)
```

- Stalemate breaks
  Other schizo ramblings

```
// What I am trying to do right now: Sac Efficiency Calculator
// What is a sac? When soldier bombs into the other team (usually for the medic or demo)
// When does this happen? On even uber situations/stalemates (x amount of time since last cap, most players are alive)
// , Disadvantaged situations

// Ways to identify stalemates:
// I guess first identify stalemates so write the analyzer
// Base case: soldier dies when all other 11 players are alive on similar uber scenarios

```

- Team fight (dry fights/mids) %
- Uber fight %
- Defensive power rating(parking the bus)/offensive power rating
- Midfight to round conversion
- Average round win length (do they win rolls or close rounds?)
- Caps per round win

- player impact rating (impact + kills + ???) similar to hltv

now that I have logs imported:
connect each log with a demo (if possible) -> done with utc timestamp
write an analyzer (similar to the collector) which will use demo parsing to get the data that I want
ideally this parser does not run on one thread and dispatches workers to parse and add because the collector is already very slow

Very Long down the road...

- Real-time score prediction/round prediction based on current gamestate
  (would need to parse demos for this data)
  also this is kind of a different type of service entirely
