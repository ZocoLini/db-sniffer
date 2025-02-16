#!/bin/bash

bash "build.sh"
bash "stop.sh"

docker run --name mssql-db-sniffer -p 1433:1433 --rm -d mssql:db-sniffer
docker run --name mysql-db-sniffer -p 3306:3306 --rm -d mysql:db-sniffer