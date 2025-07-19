package com.example.demo;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.cache.annotation.EnableCaching;
import org.springframework.scheduling.annotation.EnableAsync;

@SpringBootApplication
@EnableCaching
@EnableAsync
public class PerformanceDemoApplication {

    public static void main(String[] args) {
        // Set system properties for demo
        System.setProperty("spring.datasource.url", "jdbc:h2:mem:testdb");
        System.setProperty("spring.jpa.hibernate.ddl-auto", "create");
        System.setProperty("logging.level.org.springframework", "WARN");
        System.setProperty("logging.level.org.hibernate", "WARN");
        
        var context = SpringApplication.run(PerformanceDemoApplication.class, args);
        
        try {
            Thread.sleep(1000); // Let Spring fully initialize
            
            // Run business logic that should show up in profiling
            performBusinessLogic();
            
            Thread.sleep(1000); // Let profiler collect data
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        context.close();
    }
    
    private static void performBusinessLogic() {
        System.out.println("Performing business logic...");
        
        // CPU-intensive business operations
        calculatePrimes(1000);
        processStrings(500);
        simulateComplexCalculation();
        
        System.out.println("Business logic completed.");
    }
    
    private static void calculatePrimes(int limit) {
        // This should show up as user code
        for (int i = 2; i <= limit; i++) {
            boolean isPrime = true;
            for (int j = 2; j * j <= i; j++) {
                if (i % j == 0) {
                    isPrime = false;
                    break;
                }
            }
            if (isPrime) {
                // Simulate some work
                Math.sqrt(i);
            }
        }
    }
    
    private static void processStrings(int count) {
        // String processing that should be visible
        StringBuilder result = new StringBuilder();
        for (int i = 0; i < count; i++) {
            result.append("Processing item ").append(i);
            result.append(" with complex string operations ");
            result.append(Math.random() * 1000);
            result.append("\n");
            
            // Simulate string manipulation
            String temp = result.toString();
            temp = temp.toUpperCase().toLowerCase();
            temp.contains("item");
        }
    }
    
    private static void simulateComplexCalculation() {
        // Complex mathematical operation
        double result = 0;
        for (int i = 0; i < 10000; i++) {
            result += Math.sin(i) * Math.cos(i * 2) * Math.tan(i / 3.0);
            result = Math.sqrt(Math.abs(result));
        }
        System.out.println("Calculation result: " + result);
    }
}