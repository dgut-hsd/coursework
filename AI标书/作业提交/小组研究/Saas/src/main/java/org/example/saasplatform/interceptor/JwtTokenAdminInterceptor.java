package org.example.saasplatform.interceptor;

import com.alibaba.fastjson2.JSON;
import io.jsonwebtoken.Claims;
import io.jsonwebtoken.JwtException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.example.saasplatform.common.BaseContext;
import org.example.saasplatform.util.JwtUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.springframework.web.servlet.HandlerInterceptor;

import java.util.Map;

/**
 * Custom JWT authentication interceptor — replaces Spring Security.
 * <p>
 * Extracts Bearer token from Authorization header, validates it,
 * and sets userId + tenantId into {@link BaseContext}.
 * Cleaned up in {@code afterCompletion()}.
 * <p>
 * Using HandlerInterceptor instead of Filter:
 * - Exceptions propagate to Spring's @RestControllerAdvice
 * - Returns proper JSON error responses, not raw 500
 */
@Component
public class JwtTokenAdminInterceptor implements HandlerInterceptor {

    private static final Logger log = LoggerFactory.getLogger(JwtTokenAdminInterceptor.class);

    @Autowired
    private JwtUtil jwtUtil;

    @Override
    public boolean preHandle(HttpServletRequest request,
                             HttpServletResponse response,
                             Object handler) throws Exception {
        // 1. Extract token from Authorization header
        String token = extractToken(request);
        if (token == null || token.isEmpty()) {
            sendError(response, 401, "未登录：缺少Authorization头");
            return false;
        }

        // 2. Validate token and extract claims (parse once)
        try {
            Claims claims = jwtUtil.parseToken(token);
            Long userId = jwtUtil.getUserId(claims);
            Long tenantId = jwtUtil.getTenantId(claims);

            if (userId == null || tenantId == null) {
                sendError(response, 401, "Token无效：缺少用户或租户信息");
                return false;
            }

            // 3. Set context
            BaseContext.setCurrentUserId(userId);
            BaseContext.setCurrentTenantId(tenantId);

            log.debug("Auth OK: userId={}, tenantId={}", userId, tenantId);
            return true;

        } catch (JwtException e) {
            sendError(response, 401, "Token无效或已过期");
            return false;
        }
    }

    @Override
    public void afterCompletion(HttpServletRequest request,
                                HttpServletResponse response,
                                Object handler, Exception ex) {
        // MANDATORY: clean up ThreadLocal to prevent cross-request data leaks
        BaseContext.removeCurrentUserId();
        BaseContext.removeCurrentTenantId();
    }

    private String extractToken(HttpServletRequest request) {
        String header = request.getHeader("Authorization");
        if (header != null && header.startsWith("Bearer ")) {
            return header.substring(7);
        }
        return null;
    }

    private void sendError(HttpServletResponse response, int code, String message) throws Exception {
        response.setStatus(200); // Let the client read the JSON body
        response.setContentType("application/json;charset=UTF-8");
        response.getWriter().write(JSON.toJSONString(
                Map.of("code", code, "message", message, "timestamp", System.currentTimeMillis())
        ));
    }
}
