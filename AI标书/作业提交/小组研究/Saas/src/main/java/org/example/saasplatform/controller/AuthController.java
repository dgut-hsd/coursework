package org.example.saasplatform.controller;

import org.example.saasplatform.common.Result;
import org.example.saasplatform.dto.LoginRequest;
import org.example.saasplatform.dto.RegisterRequest;
import org.example.saasplatform.service.AuthService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.Map;

@RestController
@RequestMapping("/api/auth")
public class AuthController {

    @Autowired
    private AuthService authService;

    @PostMapping("/register")
    public Result<Map<String, Object>> register(@RequestBody RegisterRequest request) {
        Map<String, Object> data = authService.register(request.getUsername(), request.getPassword());
        return Result.success(data);
    }

    @PostMapping("/login")
    public Result<Map<String, Object>> login(@RequestBody LoginRequest request) {
        Map<String, Object> data = authService.login(request.getUsername(), request.getPassword());
        return Result.success(data);
    }
}
