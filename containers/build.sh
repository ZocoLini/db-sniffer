#!/bin/bash

docker build -t mysql:db-sniffer -f "mysql.dockerfile" .
docker build -t mssql:db-sniffer -f "mssql.dockerfile" .