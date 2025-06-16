FROM mcr.microsoft.com/mssql/server:2022-latest
USER root

COPY mssql_db_creation.sql /usr/config/mssql_db_creation.sql

COPY mssql_init.sh /usr/config/init.sh
RUN chmod +x /usr/config/init.sh

CMD ["/bin/bash", "/usr/config/init.sh"]

