package com.example.demo.service;

import com.example.demo.model.User;
import org.apache.commons.math3.util.FastMath;
import org.springframework.stereotype.Service;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;

@Service
public class BusinessLogicService {
    
    private final Pattern emailPattern = Pattern.compile("^[A-Za-z0-9+_.-]+@(.+)$");
    
    public void validateUserData(String name, String email, Integer age) {
        // Simulate complex validation logic
        validateName(name);
        validateEmail(email);
        validateAge(age);
        
        // Simulate additional business rules
        performComplexValidation(name, email, age);
    }
    
    private void validateName(String name) {
        if (name == null || name.trim().isEmpty()) {
            throw new IllegalArgumentException("Name cannot be empty");
        }
        
        // Simulate expensive validation
        for (int i = 0; i < 100; i++) {
            name.toLowerCase().contains("admin"); // Simulate security check
        }
    }
    
    private void validateEmail(String email) {
        if (email == null || !emailPattern.matcher(email).matches()) {
            throw new IllegalArgumentException("Invalid email format");
        }
        
        // Simulate domain validation
        String domain = email.substring(email.indexOf('@') + 1);
        validateDomain(domain);
    }
    
    private void validateAge(Integer age) {
        if (age == null || age < 0 || age > 150) {
            throw new IllegalArgumentException("Invalid age");
        }
    }
    
    private void validateDomain(String domain) {
        // Simulate expensive domain validation
        List<String> validDomains = List.of("example.com", "test.com", "demo.org");
        
        for (String validDomain : validDomains) {
            if (domain.contains(validDomain)) {
                return;
            }
        }
        
        // Simulate DNS lookup simulation
        try {
            Thread.sleep(2);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
    
    private void performComplexValidation(String name, String email, Integer age) {
        // Simulate complex business logic with mathematical operations
        double score = calculateUserScore(name, email, age);
        
        if (score < 0.5) {
            throw new IllegalArgumentException("User data does not meet quality standards");
        }
    }
    
    private double calculateUserScore(String name, String email, Integer age) {
        // Simulate CPU-intensive calculation
        double nameScore = 0;
        for (char c : name.toCharArray()) {
            nameScore += FastMath.sin(c) * FastMath.cos(c);
        }
        
        double emailScore = 0;
        for (char c : email.toCharArray()) {
            emailScore += FastMath.log(Math.abs(c) + 1);
        }
        
        double ageScore = FastMath.sqrt(age) / 10.0;
        
        return (nameScore + emailScore + ageScore) / 3.0;
    }
    
    public void processUserCreation(User user) {
        // Simulate post-creation processing
        generateUserProfile(user);
        calculateRecommendations(user);
        sendWelcomeNotification(user);
    }
    
    private void generateUserProfile(User user) {
        // Simulate profile generation
        StringBuilder profile = new StringBuilder();
        for (int i = 0; i < 50; i++) {
            profile.append("Profile data for ").append(user.getName()).append(" - ").append(i).append("\n");
        }
        
        // Simulate processing time
        try {
            Thread.sleep(5);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
    
    private void calculateRecommendations(User user) {
        // Simulate recommendation engine
        List<String> recommendations = new ArrayList<>();
        
        for (int i = 0; i < 20; i++) {
            String recommendation = "Recommendation " + i + " for " + user.getName();
            recommendations.add(recommendation);
            
            // Simulate complex scoring
            double score = FastMath.random() * user.getAge() * recommendation.length();
            FastMath.abs(score);
        }
    }
    
    private void sendWelcomeNotification(User user) {
        // Simulate notification sending
        String message = buildWelcomeMessage(user);
        
        // Simulate network call
        try {
            Thread.sleep(10);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
    
    private String buildWelcomeMessage(User user) {
        StringBuilder message = new StringBuilder();
        message.append("Welcome, ").append(user.getName()).append("!\n");
        message.append("Your email is: ").append(user.getEmail()).append("\n");
        message.append("Age: ").append(user.getAge()).append("\n");
        
        // Simulate template processing
        for (int i = 0; i < 10; i++) {
            message.append("Welcome line ").append(i).append("\n");
        }
        
        return message.toString();
    }
    
    public void logSearchAttempt(String pattern) {
        // Simulate logging with expensive string operations
        String logMessage = String.format("Search performed for pattern: '%s' at %d", 
                                         pattern, System.currentTimeMillis());
        
        // Simulate log processing
        for (int i = 0; i < 20; i++) {
            logMessage = logMessage.toUpperCase().toLowerCase();
        }
    }
}