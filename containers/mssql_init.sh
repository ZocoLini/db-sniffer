#!/bin/bash
set -e

/opt/mssql/bin/sqlservr &

# Espera a que SQL Server esté listo
echo "Esperando a que SQL Server se inicie..."
sleep 10

# Ejecuta el script de inicialización
echo "Ejecutando script de inicialización..."
/opt/mssql-tools18/bin/sqlcmd -S "localhost" -U SA -P "D3fault&Pass" -C -d master -i /usr/config/mssql_db_creation.sql

echo "Inicialización completada."
sleep 60