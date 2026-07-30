package org.example.saasplatform.controller;

import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.common.Result;
import org.example.saasplatform.dto.CreateTaskRequest;
import org.example.saasplatform.entity.AuditTask;
import org.example.saasplatform.service.AuditTaskService;
import org.example.saasplatform.sse.SseHub;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.util.Map;

@RestController
@RequestMapping("/api/audit-tasks")
public class AuditTaskController {

    private static final Logger log = LoggerFactory.getLogger(AuditTaskController.class);

    @Autowired
    private AuditTaskService auditTaskService;

    @Autowired
    private SseHub sseHub;

    // ── CRUD ────────────────────────────────────────────────

    @PostMapping
    public Result<AuditTask> create(@RequestBody CreateTaskRequest request) {
        AuditTask task = new AuditTask();
        task.setProjectId(request.getProjectId());
        task.setEnabledChecks(request.getEnabledChecks());
        return Result.success(auditTaskService.createTask(task));
    }

    @GetMapping("/page")
    public Result<Page<AuditTask>> page(
            @RequestParam(defaultValue = "1") int pageNum,
            @RequestParam(defaultValue = "10") int pageSize,
            @RequestParam(required = false) String status,
            @RequestParam(required = false) String startTime) {
        return Result.success(auditTaskService.pageTasks(pageNum, pageSize, status, startTime));
    }

    @GetMapping("/{id}")
    public Result<AuditTask> getById(@PathVariable Long id) {
        return Result.success(auditTaskService.getTaskById(id));
    }

    // ── SSE Stream ──────────────────────────────────────────

    /**
     * SSE real-time event stream for audit task progress.
     * <p>
     * Usage:
     * <pre>{@code
     * curl -N http://localhost:8080/api/audit-tasks/42/stream \
     *   -H "Authorization: Bearer <token>"
     *
     * # Reconnect with last event ID:
     * curl -N http://localhost:8080/api/audit-tasks/42/stream \
     *   -H "Authorization: Bearer <token>" \
     *   -H "Last-Event-ID: 15"
     * }</pre>
     */
    @GetMapping("/{taskId}/stream")
    public SseEmitter stream(
            @PathVariable Long taskId,
            @RequestHeader(value = "Last-Event-ID", required = false) String lastEventId) {
        log.info("SSE stream requested: taskId={}, lastEventId={}", taskId, lastEventId);
        return sseHub.subscribe(taskId, lastEventId);
    }

    // ── Callback (from Rust engine) ─────────────────────────

    /**
     * Callback endpoint for Rust engine to update task status and push events.
     * Excluded from JWT interceptor (configured in WebConfig).
     * In production: secured by shared API key.
     */
    @PostMapping("/callback")
    public Result<Void> callback(@RequestBody Map<String, Object> payload) {
        Long taskId = Long.valueOf(payload.get("taskId").toString());
        String status = payload.get("status").toString();
        log.info("Callback received: taskId={}, status={}", taskId, status);

        // Update task status
        auditTaskService.updateTaskStatus(taskId, status);

        // Relay events from Rust engine to SSE clients
        if (payload.containsKey("events")) {
            @SuppressWarnings("unchecked")
            var events = (java.util.List<Map<String, Object>>) payload.get("events");
            for (Map<String, Object> event : events) {
                sseHub.sendEvent(taskId,
                        event.get("type").toString(),
                        event.get("data"));
            }
        }

        return Result.success();
    }
}
