package com.example.demo.service;

import com.example.demo.model.User;
import com.example.demo.repository.UserRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.cache.annotation.Cacheable;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.concurrent.CompletableFuture;

@Service
@Transactional
public class UserService {
    
    @Autowired
    private UserRepository userRepository;
    
    @Autowired
    private BusinessLogicService businessLogicService;
    
    public User createUser(String name, String email, Integer age) {
        // Simulate validation and business logic
        businessLogicService.validateUserData(name, email, age);
        
        User user = new User(name, email, age);
        
        // Simulate some heavy processing
        businessLogicService.processUserCreation(user);
        
        return userRepository.save(user);
    }
    
    @Cacheable("users")
    public List<User> getAllUsers() {
        // Simulate expensive operation
        try {
            Thread.sleep(50); // Simulate database query time
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        return userRepository.findAll();
    }
    
    public List<User> searchUsers(String namePattern) {
        // Simulate complex search logic
        businessLogicService.logSearchAttempt(namePattern);
        
        return userRepository.findByNameContaining(namePattern);
    }
    
    @Async
    public CompletableFuture<List<User>> getUsersByAgeRangeAsync(Integer minAge, Integer maxAge) {
        // Simulate async processing
        try {
            Thread.sleep(100);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        List<User> users = userRepository.findByAgeRange(minAge, maxAge);
        return CompletableFuture.completedFuture(users);
    }
    
    public void bulkCreateUsers(int count) {
        for (int i = 0; i < count; i++) {
            String name = "User" + i;
            String email = "user" + i + "@example.com";
            Integer age = 20 + (i % 50);
            
            createUser(name, email, age);
            
            // Simulate processing delay
            if (i % 10 == 0) {
                try {
                    Thread.sleep(5);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                }
            }
        }
    }
}