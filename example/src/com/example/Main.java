package com.example;

public class Main {
    public static void main(String[] args) {
        System.out.println("Hello from Java!");
        System.out.println("Starting sleeping...");
        try {
            Thread.sleep(10000);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }
}
