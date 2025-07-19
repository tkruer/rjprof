package com.example.demo.repository;

import com.example.demo.model.User;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;

@Repository
public interface UserRepository extends JpaRepository<User, Long> {
    
    List<User> findByNameContaining(String name);
    
    List<User> findByAgeGreaterThan(Integer age);
    
    @Query("SELECT u FROM User u WHERE u.email LIKE %:domain%")
    List<User> findByEmailDomain(@Param("domain") String domain);
    
    @Query(value = "SELECT * FROM users WHERE age BETWEEN ?1 AND ?2", nativeQuery = true)
    List<User> findByAgeRange(Integer minAge, Integer maxAge);
}