package org.example.saasplatform.dispatch;

import org.example.saasplatform.entity.AuditTask;

/**
 * Strategy interface for audit task dispatch.
 * <p>
 * Three implementations switchable via {@code audit.dispatcher.type} property:
 * - async (default): @Async thread pool
 * - redis-list: Redis LPUSH / BRPOP
 * - redis-stream: Redis Streams with Consumer Group + DLQ
 */
public interface AuditTaskDispatcher {

    /**
     * Dispatch a newly created audit task for asynchronous processing.
     */
    void dispatch(AuditTask task);
}
