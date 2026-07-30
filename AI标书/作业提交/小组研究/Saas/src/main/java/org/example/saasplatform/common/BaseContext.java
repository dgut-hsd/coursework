package org.example.saasplatform.common;

/**
 * Request-scoped user/tenant context via ThreadLocal.
 * <p>
 * Must be cleaned up in {@code afterCompletion()} — Tomcat reuses threads,
 * stale context would leak tenant data between requests.
 */
public class BaseContext {

    private static final ThreadLocal<Long> USER_ID = new ThreadLocal<>();
    private static final ThreadLocal<Long> TENANT_ID = new ThreadLocal<>();

    // ── userId ──────────────────────────────────────────────

    public static void setCurrentUserId(Long id) {
        USER_ID.set(id);
    }

    public static Long getCurrentUserId() {
        return USER_ID.get();
    }

    public static void removeCurrentUserId() {
        USER_ID.remove();
    }

    // ── tenantId ────────────────────────────────────────────

    public static void setCurrentTenantId(Long id) {
        TENANT_ID.set(id);
    }

    public static Long getCurrentTenantId() {
        return TENANT_ID.get();
    }

    public static void removeCurrentTenantId() {
        TENANT_ID.remove();
    }
}
