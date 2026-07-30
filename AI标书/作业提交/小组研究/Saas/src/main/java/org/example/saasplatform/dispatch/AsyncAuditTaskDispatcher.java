package org.example.saasplatform.dispatch;

import org.example.saasplatform.common.BaseContext;
import org.example.saasplatform.entity.AuditTask;
import org.example.saasplatform.service.AuditEngineService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

/**
 * Default dispatcher — uses Spring @Async thread pool.
 * Activated when {@code audit.dispatcher.type=async} or property is absent.
 */
@Service
@ConditionalOnProperty(name = "audit.dispatcher.type", havingValue = "async", matchIfMissing = true)
public class AsyncAuditTaskDispatcher implements AuditTaskDispatcher {

    private static final Logger log = LoggerFactory.getLogger(AsyncAuditTaskDispatcher.class);

    @Autowired
    private AuditEngineService auditEngineService;

    @Async("auditTaskExecutor")
    @Override
    public void dispatch(AuditTask task) {
        log.info("Async dispatch: taskId={}", task.getId());
        try {
            // Restore tenant context in async thread (ThreadLocal from request thread is gone)
            BaseContext.setCurrentTenantId(task.getTenantId());
            auditEngineService.executeAudit(task);
        } finally {
            BaseContext.removeCurrentUserId();
            BaseContext.removeCurrentTenantId();
        }
    }
}
