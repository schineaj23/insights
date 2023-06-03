`https://api.rgl.gg/v0/`

<br><br>

# `/seasons/:id`
Inputs
- ID is seasonId

Outputs
- `name`
- `divisionSorting` List: idk what this is
- `formatName`
- `regionName`
- `maps` List
- `participatingTeams` List
- `matchesPlayedDuringSeason` List

<br><br>

# `/teams/:id`
Inputs
- `id` is teamId

Outputs
- `teamId` Number
- `linkedTeams` List
- `seasonId` Number
- `divisionId` Number
- `divisionName` String
- `teamLeader` SteamID64 String
- `createdAt` timestamp
- `updatedAt` timestamp
- `tag` String
- `name` String
- `finalRank` Number
- `players` List
    - `name` String
    - `steamId` SteamID64 String
    - `isLeader` Boolean
    - `joinedAt` Timestamp
    - `leftAt` Timestamp
