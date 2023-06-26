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
  This has to be only when people
- Stalemate breaks
- Team fight (dry fights/mids) %
- Uber fight %

- player impact rating (impact + kills + ???) similar to hltv

now that I have logs imported:
connect each log with a demo (if possible) -> done with utc timestamp
write an analyzer (similar to the collector) which will use demo parsing to get the data that I want
ideally this parser does not run on one thread and dispatches workers to parse and add because the collector is already very slow

Very Long down the road...
- Real-time score prediction/round prediction based on current gamestate 
  (would need to parse demos for this data)
  also this is kind of a different type of service entirely
