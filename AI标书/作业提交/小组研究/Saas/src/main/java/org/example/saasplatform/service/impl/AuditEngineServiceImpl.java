package org.example.saasplatform.service.impl;

import org.example.saasplatform.client.RustApiClient;
import org.example.saasplatform.entity.AuditTask;
import org.example.saasplatform.mapper.AuditTaskMapper;
import org.example.saasplatform.service.AuditEngineService;
import org.example.saasplatform.sse.SseHub;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import java.util.Map;

/**
 * Orchestrates the 4-stage audit lifecycle.
 * <p>
 * Called by the active {@code AuditTaskDispatcher} implementation.
 */
@Service
public class AuditEngineServiceImpl implements AuditEngineService {

    private static final Logger log = LoggerFactory.getLogger(AuditEngineServiceImpl.class);

    @Autowired
    private AuditTaskMapper auditTaskMapper;

    @Autowired
    private SseHub sseHub;

    @Autowired
    private RustApiClient rustApiClient;

    @Override
    public void executeAudit(AuditTask task) {
        Long taskId = task.getId();
        Long tenantId = task.getTenantId();

        try {
            // ═══ Stage 1: PROCESSING ═══
            log.info("Stage 1: taskId={} → PROCESSING", taskId);
            updateStatus(taskId, tenantId, AuditTask.STATUS_PROCESSING);
            sseHub.sendEvent(taskId, "progress",
                    Map.of("message", "审核已开始", "progress", 0, "stage", "init"));

            // ═══ Stage 2: Progress ═══
            log.info("Stage 2: taskId={} → analyzing", taskId);
            sleepMs(500);
            sseHub.sendEvent(taskId, "progress",
                    Map.of("message", "正在分析标书结构...", "progress", 30, "stage", "analyzing"));

            sleepMs(500);
            sseHub.sendEvent(taskId, "progress",
                    Map.of("message", "正在执行安全扫描...", "progress", 60, "stage", "scanning"));

            // ═══ Stage 3: Call Rust AI engine (mock) ═══
            log.info("Stage 3: taskId={} → calling Rust engine", taskId);
            sseHub.sendEvent(taskId, "progress",
                    Map.of("message", "正在调用AI引擎深度分析...", "progress", 80, "stage", "ai-analysis"));

            Map<String, Object> result = rustApiClient.simulateAudit(taskId, task.getEnabledChecks());

            // Push findings individually
            sseHub.sendEvent(taskId, "finding", result.get("findings"));

            // ═══ Stage 4: COMPLETED ═══
            log.info("Stage 4: taskId={} → COMPLETED, score={}", taskId, result.get("score"));
            updateStatus(taskId, tenantId, AuditTask.STATUS_COMPLETED);
            sseHub.sendEvent(taskId, "complete", Map.of(
                    "score", result.get("score"),
                    "message", "审核已完成",
                    "findingsCount",
                    ((java.util.List<?>) result.get("findings")).size()
            ));

        } catch (Exception e) {
            log.error("Audit failed: taskId={}", taskId, e);
            updateStatus(taskId, tenantId, AuditTask.STATUS_FAILED);
            sseHub.sendEvent(taskId, "error",
                    Map.of("message", "审核失败: " + e.getMessage()));
        }
    }

    private void updateStatus(Long taskId, Long tenantId, String status) {
        AuditTask update = new AuditTask();
        update.setId(taskId);
        update.setTenantId(tenantId);  // Required: TenantLine interceptor in async thread has no ThreadLocal
        update.setStatus(status);
        auditTaskMapper.updateById(update);
    }

    private void sleepMs(long ms) {
        try {
            Thread.sleep(ms);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
