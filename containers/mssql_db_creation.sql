create database test_db;
go

use  test_db;
go

create table Department (
    id int primary key identity(1, 1),
    name varchar(255),
    abreviation char(3),
    type char(1),
    constraint UQ_DEPARTMENT_ABRV unique (abreviation)
);
go

create table Person (
    id int primary key identity(1, 1),
    name varchar(255),
    age int,
    birthdate date,
    created datetime,
    department_id int,
    salario numeric,
    salario_extra float,
    paga_extra double precision,
    foreign key (department_id) references Department(id) -- one-to-many
);
go

create table Address (
    id int primary key identity(1, 1),
    street varchar(255),
    city varchar(255),
    postal_code varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) -- One-to-one
);
go

create table Phone (
    id int primary key identity(1, 1),
    number varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) -- One-to-many
);
go

create table Email (
    id int primary key identity(1, 1),
    email varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) -- One-to-many
);
go

create table Project (
    id int primary key identity(1, 1),
    name varchar(255)
);
go

create table Person_Project (
    person_id int,
    project_id int,
    primary key (person_id, project_id),
    foreign key (person_id) references Person(id), -- Many-to-many (Person-Project)
    foreign key (project_id) references Project(id)
);
go

-- Multiple key references
create table ComposedPKTable (
                                 fist_key int,
                                 second_key int,
                                 other_field varchar(255),
                                 primary key (fist_key, second_key)
);
go
create table ComposedFKAsPKTable (
    fist_key int,
    second_key int,
    a int,
    b int,
    other_field varchar(255),
    primary key (fist_key, second_key),
    foreign key (fist_key, second_key) references ComposedPKTable(fist_key, second_key),
    foreign key (a, b) references ComposedPKTable(fist_key, second_key)
);
go
create table ComposedFKTable (
                                 id int primary key identity(1, 1),
                                 fist_key int,
                                 second_key int,
                                 a int,
                                 b int,
                                 foreign key (fist_key, second_key) references ComposedPKTable(fist_key, second_key)

);
go

-- Insert Department data (many-to-one with Person)
INSERT INTO Department (name, abreviation) VALUES
                                  ('Engineering', 'ENG'),
                                  ('Marketing', 'MKT'),
                                  ('Development', 'DEV'),
                                  ('Human Resources', 'HRE'),
                                  ('Finance', 'FIN');
go
-- Insert Person data
INSERT INTO Person (name, age, birthdate, created, department_id, salario) VALUES
                                                                      ('John Smith', 35, '1989-03-15', GETDATE(), 1, 20.90),
                                                                      ('Emma Wilson', 28, '1996-07-22', GETDATE(), 2,20.90),
                                                                      ('Michael Brown', 42, '1982-11-30', GETDATE(), 2, 20.90),
                                                                      ('Sarah Davis', 31, '1993-05-08', GETDATE(), 3, 20.90),
                                                                      ('James Johnson', 45, '1979-09-14', GETDATE(), 4, 20.90);

go
-- Insert Address data (one-to-one with Person)
INSERT INTO Address (street, city, postal_code, person_id) VALUES
                                                               ('123 Main St', 'New York', '10001', 1),
                                                               ('456 Oak Ave', 'Los Angeles', '90001', 2),
                                                               ('789 Pine Rd', 'Chicago', '60601', 3),
                                                               ('321 Maple Dr', 'Houston', '77001', 4),
                                                               ('654 Cedar Ln', 'Phoenix', '85001', 5);
go
-- Insert Phone data (one-to-many with Person)
INSERT INTO Phone (number, person_id) VALUES
                                          ('555-0101', 1),
                                          ('555-0102', 1),
                                          ('555-0201', 2),
                                          ('555-0301', 3),
                                          ('555-0302', 3),
                                          ('555-0401', 4),
                                          ('555-0501', 5);
go
-- Insert Email data (one-to-many with Person)
INSERT INTO Email (email, person_id) VALUES
                                         ('john.smith@email.com', 1),
                                         ('john.work@email.com', 1),
                                         ('emma.wilson@email.com', 2),
                                         ('michael.brown@email.com', 3),
                                         ('michael.b@work.com', 3),
                                         ('sarah.davis@email.com', 4),
                                         ('james.johnson@email.com', 5);
go
-- Insert Project data
INSERT INTO Project (name)VALUES
                               ('Website Redesign'),
                               ('Mobile App Development'),
                               ('Data Migration'),
                               ('Cloud Infrastructure'),
                               ('Marketing Campaign');
go
-- Insert Person_Project relationships (many-to-many)
INSERT INTO Person_Project (person_id, project_id)
VALUES (1, 1),
       (1, 2),
       (2, 5),
       (3, 2),
       (3, 3),
       (4, 4),
       (5, 3),
       (5, 4);
go