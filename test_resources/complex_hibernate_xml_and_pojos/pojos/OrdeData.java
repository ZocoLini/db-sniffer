
package POJOS;

import java.text.SimpleDateFormat;
import java.util.Comparator;
import java.util.Date;

/**@author María José Galán López */
public class OrdeData implements Comparator <Date>{
SimpleDateFormat formato = new SimpleDateFormat("dd/MM/yyyy");
    @Override
    public int compare(Date d1, Date d2) {
       return formato.format(d2).compareTo(formato.format(d1));
    }
    
}
