package org.example.saasplatform.util;

import io.jsonwebtoken.Claims;
import io.jsonwebtoken.JwtException;
import io.jsonwebtoken.Jwts;
import io.jsonwebtoken.security.Keys;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import javax.crypto.SecretKey;
import java.nio.charset.StandardCharsets;
import java.util.Date;

/**
 * JWT utility — create, parse, and validate tokens.
 * Uses jjwt 0.12.x API.
 */
@Component
public class JwtUtil {

    @Value("${jwt.secret}")
    private String secret;

    @Value("${jwt.expiration}")
    private long expiration;

    private SecretKey getSigningKey() {
        return Keys.hmacShaKeyFor(secret.getBytes(StandardCharsets.UTF_8));
    }

    /**
     * Create a JWT token containing userId (subject) and tenantId (claim).
     */
    public String createToken(Long userId, Long tenantId) {
        return Jwts.builder()
                .subject(userId.toString())
                .claim("tenantId", tenantId)
                .issuedAt(new Date())
                .expiration(new Date(System.currentTimeMillis() + expiration))
                .signWith(getSigningKey())
                .compact();
    }

    /**
     * Parse token and return claims. Throws JwtException on invalid/expired token.
     */
    public Claims parseToken(String token) {
        return Jwts.parser()
                .verifyWith(getSigningKey())
                .build()
                .parseSignedClaims(token)
                .getPayload();
    }

    /**
     * Extract userId from parsed claims (avoids re-parsing).
     */
    public Long getUserId(Claims claims) {
        return Long.parseLong(claims.getSubject());
    }

    /**
     * Extract tenantId from parsed claims (avoids re-parsing).
     */
    public Long getTenantId(Claims claims) {
        return claims.get("tenantId", Long.class);
    }

    /**
     * Validate token signature and expiration.
     */
    public boolean validateToken(String token) {
        try {
            parseToken(token);
            return true;
        } catch (JwtException | IllegalArgumentException e) {
            return false;
        }
    }
}
