package POJOS;
// Generated 02-feb-2021 0:15:37 by Hibernate Tools 3.6.0

import java.util.Objects;




/**
 * EmpregadoProxecto generated by hbm2java
 */
public class EmpregadoProxecto  implements java.io.Serializable {


     private EmpregadoProxectoId id;
     private Empregado empregado;
     private Proxecto proxecto;
     private Integer horas;

    public EmpregadoProxecto() {
    }

    public EmpregadoProxecto(EmpregadoProxectoId id, Integer horas) {
        this.id = id;
        this.horas = horas;
    }

	
    public EmpregadoProxecto(EmpregadoProxectoId id, Empregado empregado, Proxecto proxecto) {
        this.id = id;
        this.empregado = empregado;
        this.proxecto = proxecto;
    }
    public EmpregadoProxecto(EmpregadoProxectoId id, Empregado empregado, Proxecto proxecto, Integer horas) {
       this.id = id;
       this.empregado = empregado;
       this.proxecto = proxecto;
       this.horas = horas;
    }

    public EmpregadoProxecto(EmpregadoProxectoId id) {
        this.id = id;
    }
    
   
    public EmpregadoProxectoId getId() {
        return this.id;
    }
    
    public void setId(EmpregadoProxectoId id) {
        this.id = id;
    }
    public Empregado getEmpregado() {
        return this.empregado;
    }
    
    public void setEmpregado(Empregado empregado) {
        this.empregado = empregado;
    }
    public Proxecto getProxecto() {
        return this.proxecto;
    }
    
    public void setProxecto(Proxecto proxecto) {
        this.proxecto = proxecto;
    }
    public Integer getHoras() {
        return this.horas;
    }
    
    public void setHoras(Integer horas) {
        this.horas = horas;
    }

    @Override
    public int hashCode() {
        int hash = 7;
        hash = 19 * hash + Objects.hashCode(this.id);
        return hash;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == null) {
            return false;
        }
        if (getClass() != obj.getClass()) {
            return false;
        }
        final EmpregadoProxecto other = (EmpregadoProxecto) obj;
        if (!Objects.equals(this.id, other.id)) {
            return false;
        }
        return true;
    }




}

