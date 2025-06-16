FROM mysql:latest

COPY mysql_db_creation.sql /docker-entrypoint-initdb.d/

CMD ["mysqld"]
