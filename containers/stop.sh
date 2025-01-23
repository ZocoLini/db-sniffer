#!/bin/bash

docker stop mssql-db-sniffer
docker stop mysql-db-sniffer

docker rm mssql-db-sniffer
docker rm mysql-db-sniffer