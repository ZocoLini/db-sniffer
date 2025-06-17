# DB Sniffer

db-sniffer is a tool for introspecting databases and output the results in diffenrent formats. 
This is done by 'sniffing' the database, converting the data to an intermediate structure,
and then using a generator to output the introspected data as requested.

## Usage

First of all, we need a database to sniff. Right now, db-sniffer supports sniffing MySQL and MS SQL Server databases.
(MariaDb should also work, but is not tested yet). Let's assume we have a MySQL database running on `localhost` 
with the name `test_db`, and a user `test_user` with password `abc123.`.

To sniff the database, we can use the following command:

```bash
sniffer sniff \
  -u mysql://test_user:abc123.@localhost:3306/test_db \
  -m 1 \
  -o src/main/java/com/example/entities
```

- **-m option** specifies the mode to use. This 1 creates Hibernate XML mapping files for the entitie
that will be also generated. 
- **-o option** specifies the ouput and, in general, it is the path where the
generated files will be stored. Note that, when generating .java files, the ouput path should
containd a `src` or `java` folder so the tool can detect the package structure. 
- **-u option** specifies the database connection string. The format is `db_type://user:password@host:port/db_name`.
  - For MySQL the valid db_type is `mysql`.
  - For MariaDB the valid db_type is `mariadb`.
  - For MS SQL Server the valid db_type are `mssql` and `sqlserver`.

To display the help message, you can use the following command:

```bash
sniffer help
```