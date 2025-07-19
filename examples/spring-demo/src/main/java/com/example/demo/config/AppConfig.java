package com.example.demo.config;

import org.springframework.boot.CommandLineRunner;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.cache.CacheManager;
import org.springframework.cache.concurrent.ConcurrentMapCacheManager;
import org.springframework.scheduling.concurrent.ThreadPoolTaskExecutor;

import com.example.demo.service.UserService;

import java.util.concurrent.Executor;

@Configuration
public class AppConfig {
    
    @Bean
    public CacheManager cacheManager() {
        return new ConcurrentMapCacheManager("users");
    }
    
    @Bean
    public Executor taskExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(2);
        executor.setMaxPoolSize(5);
        executor.setQueueCapacity(100);
        executor.setThreadNamePrefix("async-");
        executor.initialize();
        return executor;
    }
    
    @Bean
    public CommandLineRunner dataLoader(UserService userService) {
        return args -> {
            // Pre-load some data for testing
            System.out.println("Loading initial data...");
            
            // Create some users to trigger various code paths
            userService.createUser("Alice Johnson", "alice@example.com", 25);
            userService.createUser("Bob Smith", "bob@test.com", 30);
            userService.createUser("Charlie Brown", "charlie@demo.org", 35);
            
            // Bulk create to stress test
            userService.bulkCreateUsers(50);
            
            // Perform some operations that will show up in profiling
            userService.getAllUsers(); // Cache miss
            userService.getAllUsers(); // Cache hit
            
            userService.searchUsers("Alice");
            userService.searchUsers("Bob");
            
            // Trigger async operations
            userService.getUsersByAgeRangeAsync(20, 40);
            userService.getUsersByAgeRangeAsync(30, 50);
            
            System.out.println("Initial data loaded successfully!");
        };
    }
}