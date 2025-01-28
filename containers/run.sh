#!/bin/bash

bash "build.sh"

docker run --name mssql-db-sniffer -p 1433:1433 --replace --rm -d mssql:db-sniffer
docker run --name mysql-db-sniffer -p 3306:3306 --replace --rm -d mysql:db-sniffer