package org.example.saasplatform.service;

import java.util.Map;

public interface AuthService {

    /**
     * Register a new user — auto-creates a tenant.
     * @return JWT token + user info
     */
    Map<String, Object> register(String username, String rawPassword);

    /**
     * Login with username + password.
     * @return JWT token + user info
     */
    Map<String, Object> login(String username, String rawPassword);
}
