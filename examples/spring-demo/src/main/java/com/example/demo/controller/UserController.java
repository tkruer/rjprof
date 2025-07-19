package com.example.demo.controller;

import com.example.demo.model.User;
import com.example.demo.service.UserService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.List;
import java.util.concurrent.CompletableFuture;

@RestController
@RequestMapping("/api/users")
public class UserController {
    
    @Autowired
    private UserService userService;
    
    @PostMapping
    public ResponseEntity<User> createUser(@RequestBody CreateUserRequest request) {
        User user = userService.createUser(request.getName(), request.getEmail(), request.getAge());
        return ResponseEntity.ok(user);
    }
    
    @GetMapping
    public ResponseEntity<List<User>> getAllUsers() {
        List<User> users = userService.getAllUsers();
        return ResponseEntity.ok(users);
    }
    
    @GetMapping("/search")
    public ResponseEntity<List<User>> searchUsers(@RequestParam String q) {
        List<User> users = userService.searchUsers(q);
        return ResponseEntity.ok(users);
    }
    
    @GetMapping("/age-range")
    public CompletableFuture<ResponseEntity<List<User>>> getUsersByAgeRange(
            @RequestParam Integer minAge, 
            @RequestParam Integer maxAge) {
        
        return userService.getUsersByAgeRangeAsync(minAge, maxAge)
                .thenApply(users -> ResponseEntity.ok(users));
    }
    
    @PostMapping("/bulk")
    public ResponseEntity<String> bulkCreateUsers(@RequestParam int count) {
        userService.bulkCreateUsers(count);
        return ResponseEntity.ok("Created " + count + " users");
    }
    
    // Inner class for request body
    public static class CreateUserRequest {
        private String name;
        private String email;
        private Integer age;
        
        // Getters and setters
        public String getName() { return name; }
        public void setName(String name) { this.name = name; }
        
        public String getEmail() { return email; }
        public void setEmail(String email) { this.email = email; }
        
        public Integer getAge() { return age; }
        public void setAge(Integer age) { this.age = age; }
    }
}