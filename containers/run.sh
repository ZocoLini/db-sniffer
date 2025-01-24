#!/bin/bash

bash "build.sh"

docker run --name mssql-db-sniffer -p 1433:1433 -d mssql:db-sniffer
docker run --name mysql-db-sniffer -p 3306:3306 -d mysql:db-sniffer