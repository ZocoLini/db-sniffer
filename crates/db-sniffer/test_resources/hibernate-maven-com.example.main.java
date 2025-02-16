package com.example;

import com.example.model.*;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;

import java.util.List;

import static org.junit.Assert.*;

public class Main
{
    public static void main(String[] args)
    {
        Configuration configuration = new Configuration();
        configuration.configure();

        SessionFactory sessionFactory = null;
        Session session = null;

        try
        {
            sessionFactory = configuration.buildSessionFactory();

            // This method will query the database using hibernate to verify the correctness of the mapping
            session = sessionFactory.openSession();
            assertQueries(session);
            session.close();

            // This method will update the database using hibernate to verify the correctness of the mapping
            session = sessionFactory.openSession();
            updates(session);
            session.close();

            // This method will query the database using hibernate to verify the correctness of the updates
            session = sessionFactory.openSession();
            verifyUpdates(session);

        }
        catch (Exception exception)
        {
            exception.printStackTrace();
            fail("No exception should be thrown");
        }
        finally
        {
            if (session != null) session.close();
            if (sessionFactory != null) sessionFactory.close();
        }
    }

    private static void assertQueries(Session session)
    {
        assertEquals(4, ComposedFKAsPKTable.class.getDeclaredFields().length);
        
        Person person = session.get(Person.class, 1);

        assertEquals("John Smith", person.getName());
        assertEquals("Engineering", person.getDepartment().getName());
        assertEquals(2, person.getPersonProjects().size());

        person.getAddress();
        person.getEmails();
        person.getPhones();
        person.getSalario();
        
        final var id = new PersonProjectId();

        id.setPersonId(1);
        id.setProjectId(1);

        PersonProject personProject = session.get(PersonProject.class, id);

        assertEquals("John Smith", personProject.getPerson().getName());
        assertEquals("Website Redesign", personProject.getProject().getName());

        Department department = session.get(Department.class, 2);
        assertEquals("Marketing", department.getName());
        assertEquals(2, department.getPersons().size());

        List<Project> project =
                session.createQuery("from Project p where size(p.personProjects) = 2", Project.class).list();
        assertEquals(3, project.size());

        int subProjectsCount = session.createQuery("from SubProject s", SubProject.class).list().size();
        assertEquals(5, subProjectsCount);

        final var composedFKAsPKTableId = new ComposedFKAsPKTableId();
        composedFKAsPKTableId.setFistKey(1);
        composedFKAsPKTableId.setSecondKey(1);
        ComposedFKAsPKTable composedFKAsPKTable = session.get(ComposedFKAsPKTable.class, composedFKAsPKTableId);
        
        assertNull(composedFKAsPKTable);
        
        System.out.println("All queries are correct");
    }

    private static void updates(Session session)
    {
        session.beginTransaction();

        Person person = session.get(Person.class, 1);
        person.setName("Test");
        person.getAddress().setCity("Test");

        List<Project> projects =
                session.createQuery("from Project p where size(p.personProjects) = 2", Project.class).list();
        projects.forEach(p -> p.setName("Test"));

        projects =
                session
                        .createQuery("from Project p where p.id not in " +
                                "(select pp.project.id from PersonProject pp where pp.person.id = :personId)", Project.class)
                        .setParameter("personId", person.getId())
                        .list();

        projects.forEach(p -> {
            PersonProject personProject = new PersonProject();
            personProject.setPerson(person);
            personProject.setProject(p);

            PersonProjectId id = new PersonProjectId();
            id.setPersonId(person.getId());
            id.setProjectId(p.getId());

            personProject.setId(id);

            session.persist(personProject);
        });

        SubProject subProject = session.get(SubProject.class, 1);
        session.remove(subProject);
        
        ComposedPKTable firstComposedPKTable = new ComposedPKTable();
        ComposedPKTableId composedPKTableId = new ComposedPKTableId();
        composedPKTableId.setFistKey(1);
        composedPKTableId.setSecondKey(1);
        firstComposedPKTable.setId(composedPKTableId);
        firstComposedPKTable.setOtherField("Hello");
        
        ComposedPKTable secondComposedPKTable = new ComposedPKTable();
        ComposedPKTableId composedPKTableId2 = new ComposedPKTableId();
        composedPKTableId2.setFistKey(1);
        composedPKTableId2.setSecondKey(2);
        secondComposedPKTable.setId(composedPKTableId2);
        secondComposedPKTable.setOtherField("Bye");
        
        session.persist(firstComposedPKTable);
        session.persist(secondComposedPKTable);
        
        ComposedFKAsPKTable composedFKAsPKTable = new ComposedFKAsPKTable();
        ComposedFKAsPKTableId composedFKAsPKTableId = new ComposedFKAsPKTableId();
        composedFKAsPKTableId.setFistKey(1);
        composedFKAsPKTableId.setSecondKey(1);
        composedFKAsPKTable.setId(composedFKAsPKTableId);
        composedFKAsPKTable.setComposedPKTable(secondComposedPKTable);
        composedFKAsPKTable.setOtherField("Nice");
        
        session.persist(composedFKAsPKTable);
                
        session.getTransaction().commit();

        System.out.println("All updates are correct");
    }

    private static void verifyUpdates(Session session)
    {
        Person person = session.get(Person.class, 1);

        assertEquals("Test", person.getName());
        assertEquals("Test", person.getAddress().getCity());

        List<Project> projects = session
                        .createQuery("from Project p where p.id not in " +
    "(select pp.project.id from PersonProject pp where pp.person.id = :personId)", Project.class)
                        .setParameter("personId", person.getId())
                        .list();
        
        assertEquals(0, projects.size());

        int subProjectsCount = session.createQuery("from SubProject s", SubProject.class).list().size();
        assertEquals(4, subProjectsCount);
        
        ComposedFKAsPKTableId composedFKAsPKTableId = new ComposedFKAsPKTableId();
        composedFKAsPKTableId.setFistKey(1);
        composedFKAsPKTableId.setSecondKey(1);
        ComposedFKAsPKTable composedFKAsPKTable = session.get(ComposedFKAsPKTable.class, composedFKAsPKTableId);
        
        assertEquals("Nice", composedFKAsPKTable.getOtherField());
        
        System.out.println("All updates checks are correct");
    }
}