# Usa la imagen oficial de SQL Server
FROM mcr.microsoft.com/mssql/server:2022-latest

# Configura la contraseña de 'sa' (usuario administrador predeterminado)
ENV ACCEPT_EULA=Y
ENV MSSQL_SA_PASSWORD=D3fault&Pass

USER root

# Copia un archivo SQL para inicializar la base de datos
# Cambia 'mysql_db_creation.sql' por tu archivo SQL
COPY mssql_db_creation.sql /usr/config/mssql_db_creation.sql

# Copia un script para inicializar la base de datos al inicio del contenedor
COPY mssql_init.sh /usr/config/init.sh
RUN chmod +x /usr/config/init.sh

EXPOSE 1433

# Establece el script de inicio
CMD ["/bin/bash", "/usr/config/init.sh"]

