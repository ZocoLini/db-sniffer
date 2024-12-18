package POJOS;
// Generated 02-feb-2021 0:15:37 by Hibernate Tools 3.6.0


import java.util.ArrayList;
import java.util.Collection;
import java.util.HashSet;
import java.util.Set;

/**
 * Departamento generated by hbm2java
 */
public class Departamento  implements java.io.Serializable {


     private int numDepartamento;
     private Empregado director;
     private String nomeDepartamento;
     private Set <Empregado> empregados = new HashSet();
     private Collection <String> lugares = new ArrayList();
     private Collection <Proxecto> proxectos = new ArrayList( );

    public Departamento() {
    }

	
    public Departamento(int numDepartamento, String nomeDepartamento) {
        this.numDepartamento = numDepartamento;
        this.nomeDepartamento = nomeDepartamento;
    }

    public Departamento(int numDepartamento, Empregado director, String nomeDepartamento) {
        this.numDepartamento = numDepartamento;
        this.director = director;
        this.nomeDepartamento = nomeDepartamento;
    }

    public int getNumDepartamento() {
        return numDepartamento;
    }

    public void setNumDepartamento(int numDepartamento) {
        this.numDepartamento = numDepartamento;
    }

    public Empregado getDirector() {
        return director;
    }

    public void setDirector(Empregado director) {
        this.director = director;
    }

    public String getNomeDepartamento() {
        return nomeDepartamento;
    }

    public void setNomeDepartamento(String nomeDepartamento) {
        this.nomeDepartamento = nomeDepartamento;
    }

    public Set<Empregado> getEmpregados() {
        return empregados;
    }

    public void setEmpregados(Set<Empregado> empregados) {
        this.empregados = empregados;
    }

    public Collection<String> getLugares() {
        return lugares;
    }

    public void setLugares(Collection<String> lugares) {
        this.lugares = lugares;
    }

    public Collection<Proxecto> getProxectos() {
        return proxectos;
    }

    public void setProxectos(Collection<Proxecto> proxectos) {
        this.proxectos = proxectos;
    }

   



}


