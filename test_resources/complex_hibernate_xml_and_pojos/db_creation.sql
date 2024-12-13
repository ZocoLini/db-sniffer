CREATE DATABASE IF NOT EXISTS Test;
USE Test;

CREATE TABLE CURSO (
                       Codigo INT NOT NULL AUTO_INCREMENT,
                       Nome VARCHAR(30) NOT NULL UNIQUE,
                       Horas INT,
                       PRIMARY KEY (Codigo)
);

CREATE TABLE DEPARTAMENTO (
                              NumDepartamento INT NOT NULL AUTO_INCREMENT,
                              NSSDirector VARCHAR(15) NOT NULL,
                              NomeDepartamento VARCHAR(25) NOT NULL UNIQUE,
                              PRIMARY KEY (NumDepartamento)
);

CREATE TABLE EDICION (
                         Codigo INT NOT NULL,
                         Numero INT NOT NULL,
                         Profesor VARCHAR(15) NOT NULL,
                         Data DATE,
                         Lugar VARCHAR(25),
                         PRIMARY KEY (Codigo, Numero)
);

CREATE TABLE EMPREGADO (
                           NSS VARCHAR(15) NOT NULL,
                           NSSSupervisa VARCHAR(15),
                           NumDepartamentoPertenece INT,
                           Nome VARCHAR(25),
                           Apelido1 VARCHAR(25) NOT NULL,
                           Apelido2 VARCHAR(25),
                           Rua VARCHAR(30),
                           Numero_Calle INT,
                           Piso VARCHAR(4),
                           CP VARCHAR(5),
                           Localidade VARCHAR(25),
                           Provincia VARCHAR(15),
                           DataNacemento DATE,
                           Sexo CHAR(1),
                           PRIMARY KEY (NSS)
);

CREATE TABLE EMPREGADO_PROXECTO (
                                    NSSEmpregado VARCHAR(15) NOT NULL,
                                    NumProxecto INT NOT NULL,
                                    Horas INT,
                                    PRIMARY KEY (NSSEmpregado, NumProxecto)
);

CREATE TABLE PROXECTO (
                          NumProxecto INT NOT NULL,
                          NumDepartControla INT NOT NULL,
                          NomeProxecto VARCHAR(25) NOT NULL UNIQUE,
                          Lugar VARCHAR(25) NOT NULL,
                          PRIMARY KEY (NumProxecto)
);

CREATE TABLE VEHICULO (
                          NSS VARCHAR(15) NOT NULL,
                          Matricula VARCHAR(10),
                          Marca VARCHAR(15),
                          Modelo VARCHAR(25),
                          DataCompra DATE,
                          PRIMARY KEY (NSS)
);

ALTER TABLE DEPARTAMENTO ADD CONSTRAINT FK_DEPARTAMENTO_DIRECTOR FOREIGN KEY (NSSDirector) REFERENCES EMPREGADO(NSS);
ALTER TABLE EDICION ADD CONSTRAINT FK_EDICION_PROFESOR FOREIGN KEY (Profesor) REFERENCES EMPREGADO(NSS);
ALTER TABLE EMPREGADO ADD CONSTRAINT FK_EMPREGADO_SUPERVISOR FOREIGN KEY (NSSSupervisa) REFERENCES EMPREGADO(NSS);
ALTER TABLE EMPREGADO ADD CONSTRAINT FK_EMPREGADO_DEPARTAMENTO FOREIGN KEY (NumDepartamentoPertenece) REFERENCES DEPARTAMENTO(NumDepartamento);
ALTER TABLE EMPREGADO_PROXECTO ADD CONSTRAINT FK_EMPREGADO_PROXECTO_EMPREGADO FOREIGN KEY (NSSEmpregado) REFERENCES EMPREGADO(NSS);
ALTER TABLE EMPREGADO_PROXECTO ADD CONSTRAINT FK_EMPREGADO_PROXECTO_PROXECTO FOREIGN KEY (NumProxecto) REFERENCES PROXECTO(NumProxecto);
ALTER TABLE PROXECTO ADD CONSTRAINT FK_PROXECTO_DEPARTAMENTO FOREIGN KEY (NumDepartControla) REFERENCES DEPARTAMENTO(NumDepartamento);
ALTER TABLE VEHICULO ADD CONSTRAINT FK_VEHICULO_EMPREGADO FOREIGN KEY (NSS) REFERENCES EMPREGADO(NSS);