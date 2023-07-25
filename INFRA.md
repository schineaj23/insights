# Current Infrastructure
Database: Using aiven (only 5gb max storage) NOTE: you must use the remote url instead of localhost for the FDW, and sslmode=require
Currently I pg_restore into the dev db, remove the unneeded tables, then pg_dump it and restore that into the prod db
This process is very manual, and I could probably considerably speed it up by making it into scripts, but I am not doing that rn lmao

Possibly going to use AWS Lambda for some of the functions (importing, etc), not sure where I am going to store them though

TODO: migrate logs to tag the seasonId and divisionId