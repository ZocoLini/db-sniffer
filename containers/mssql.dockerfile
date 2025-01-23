# Usa la imagen oficial de SQL Server
FROM mcr.microsoft.com/mssql/server:latest

# Configura la contraseña de 'sa' (usuario administrador predeterminado)
ENV ACCEPT_EULA=Y
ENV SA_PASSWORD=D3fault&Pass

# Expone el puerto 1433 (el puerto predeterminado de SQL Server)
EXPOSE 1433

# Copia un archivo SQL para inicializar la base de datos
# Cambia 'db_creation.sql' por tu archivo SQL
COPY db_creation.sql /usr/config/db_creation.sql

# Copia un script para inicializar la base de datos al inicio del contenedor
COPY mssql_init.sh /usr/config/init.sh

# Establece el script de inicio
CMD ["/bin/bash", "/usr/config/init.sh"]

