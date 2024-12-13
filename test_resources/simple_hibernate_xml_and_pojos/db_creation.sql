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