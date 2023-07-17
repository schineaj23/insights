#!/bin/sh

echo "Downloading Latest Dump from demos.tf"
curl https://freezer.demos.tf/database/demostf.sql.gz -O

echo "Unzipping database"
gzip -d demostf.sql.gz

echo "Dropping Previous DB"
dropdb -U cat insights-dev-demos 
createdb insights-dev-demos

echo "Importing into DB"
psql -U cat -d insights-dev-demos < demostf.sql

echo "Cleaning up"
rm demostf.sql
