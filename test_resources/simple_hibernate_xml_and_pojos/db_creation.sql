create table Person (
    id int primary key auto_increment,
    name varchar(255),
    age int,
    birthdate date,
    created timestamp
);

create table Address (
    id int primary key auto_increment,
    street varchar(255),
    city varchar(255),
    postal_code varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) # One-to-one
);

create table Phone (
    id int primary key auto_increment,
    number varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) # One-to-many
);

create table Email (
    id int primary key auto_increment,
    email varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) # One-to-many
);

create table Department (
    id int primary key auto_increment,
    name varchar(255),
    person_id int,
    foreign key (person_id) references Person(id) # Many-to-one
);

create table Project (
    id int primary key auto_increment,
    name varchar(255)
);

create table Person_Project (
    person_id int,
    project_id int,
    primary key (person_id, project_id),
    foreign key (person_id) references Person(id), # Many-to-many (Person-Project)
    foreign key (project_id) references Project(id)
);

-- Insert Person data
INSERT INTO Person (name, age, birthdate, created) VALUES
                                                       ('John Smith', 35, '1989-03-15', NOW()),
                                                       ('Emma Wilson', 28, '1996-07-22', NOW()),
                                                       ('Michael Brown', 42, '1982-11-30', NOW()),
                                                       ('Sarah Davis', 31, '1993-05-08', NOW()),
                                                       ('James Johnson', 45, '1979-09-14', NOW());

-- Insert Address data (one-to-one with Person)
INSERT INTO Address (street, city, postal_code, person_id) VALUES
                                                               ('123 Main St', 'New York', '10001', 1),
                                                               ('456 Oak Ave', 'Los Angeles', '90001', 2),
                                                               ('789 Pine Rd', 'Chicago', '60601', 3),
                                                               ('321 Maple Dr', 'Houston', '77001', 4),
                                                               ('654 Cedar Ln', 'Phoenix', '85001', 5);

-- Insert Phone data (one-to-many with Person)
INSERT INTO Phone (number, person_id) VALUES
                                          ('555-0101', 1),
                                          ('555-0102', 1),
                                          ('555-0201', 2),
                                          ('555-0301', 3),
                                          ('555-0302', 3),
                                          ('555-0401', 4),
                                          ('555-0501', 5);

-- Insert Email data (one-to-many with Person)
INSERT INTO Email (email, person_id) VALUES
                                         ('john.smith@email.com', 1),
                                         ('john.work@email.com', 1),
                                         ('emma.wilson@email.com', 2),
                                         ('michael.brown@email.com', 3),
                                         ('michael.b@work.com', 3),
                                         ('sarah.davis@email.com', 4),
                                         ('james.johnson@email.com', 5);

-- Insert Department data (many-to-one with Person)
INSERT INTO Department (name, person_id) VALUES
                                             ('Engineering', 1),
                                             ('Marketing', 2),
                                             ('Engineering', 3),
                                             ('Human Resources', 4),
                                             ('Finance', 5);

-- Insert Project data
INSERT INTO Project (name) VALUES
                               ('Website Redesign'),
                               ('Mobile App Development'),
                               ('Data Migration'),
                               ('Cloud Infrastructure'),
                               ('Marketing Campaign');

-- Insert Person_Project relationships (many-to-many)
INSERT INTO Person_Project (person_id, project_id) VALUES
                                                       (1, 1), -- John works on Website Redesign
                                                       (1, 2), -- John also works on Mobile App
                                                       (2, 5), -- Emma works on Marketing Campaign
                                                       (3, 2), -- Michael works on Mobile App
                                                       (3, 3), -- Michael also works on Data Migration
                                                       (4, 4), -- Sarah works on Cloud Infrastructure
                                                       (5, 3), -- James works on Data Migration
                                                       (5, 4); -- James also works on Cloud Infrastructure