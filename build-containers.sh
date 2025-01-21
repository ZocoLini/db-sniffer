#!/bin/bash

cd test_resources || exit

docker build -t mysql:db-sniffer -f "mysql.dockerfile" .