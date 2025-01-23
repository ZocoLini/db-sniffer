#!/bin/bash
# Inicia el servidor SQL en segundo plano
/opt/mssql/bin/sqlservr &

# Espera a que SQL Server esté listo
echo "Esperando a que SQL Server se inicie..."
until /opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P StrongPassword123! -Q "SELECT 1" &>/dev/null; do
    sleep 1
done

echo "SQL Server está listo. Ejecutando script de inicialización..."

# Ejecuta el script SQL
/opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P StrongPassword123! -d master -i /usr/config/db_creation.sql

echo "Script de inicialización ejecutado correctamente."

# Mantiene el contenedor en ejecución
wait
