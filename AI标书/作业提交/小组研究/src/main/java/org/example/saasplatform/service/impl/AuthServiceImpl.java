package org.example.saasplatform.service.impl;

import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import org.example.saasplatform.common.BizException;
import org.example.saasplatform.entity.SysUser;
import org.example.saasplatform.mapper.SysUserMapper;
import org.example.saasplatform.service.AuthService;
import org.example.saasplatform.util.JwtUtil;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.crypto.bcrypt.BCryptPasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.LinkedHashMap;
import java.util.Map;

@Service
public class AuthServiceImpl implements AuthService {

    @Autowired
    private SysUserMapper sysUserMapper;

    @Autowired
    private JwtUtil jwtUtil;

    private final BCryptPasswordEncoder passwordEncoder = new BCryptPasswordEncoder();

    @Override
    @Transactional
    public Map<String, Object> register(String username, String rawPassword) {
        // Check uniqueness
        Long count = sysUserMapper.selectCount(
                new LambdaQueryWrapper<SysUser>().eq(SysUser::getUsername, username));
        if (count > 0) {
            throw new BizException(400, "用户名已存在");
        }

        // Auto-generate tenantId (simple: use timestamp-based)
        Long tenantId = System.currentTimeMillis() % 1000000 + 1000;

        // Create user
        SysUser user = new SysUser();
        user.setUsername(username);
        user.setPasswordHash(passwordEncoder.encode(rawPassword));
        user.setTenantId(tenantId);
        user.setCreatedAt(LocalDateTime.now());
        sysUserMapper.insert(user);

        // Issue JWT
        String token = jwtUtil.createToken(user.getId(), tenantId);

        return buildAuthResponse(token, user);
    }

    @Override
    public Map<String, Object> login(String username, String rawPassword) {
        SysUser user = sysUserMapper.selectOne(
                new LambdaQueryWrapper<SysUser>().eq(SysUser::getUsername, username));
        if (user == null) {
            throw new BizException(401, "用户名或密码错误");
        }

        if (!passwordEncoder.matches(rawPassword, user.getPasswordHash())) {
            throw new BizException(401, "用户名或密码错误");
        }

        String token = jwtUtil.createToken(user.getId(), user.getTenantId());

        return buildAuthResponse(token, user);
    }

    private Map<String, Object> buildAuthResponse(String token, SysUser user) {
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("token", token);
        Map<String, Object> userInfo = new LinkedHashMap<>();
        userInfo.put("id", user.getId());
        userInfo.put("username", user.getUsername());
        userInfo.put("tenantId", user.getTenantId());
        result.put("user", userInfo);
        return result;
    }
}
