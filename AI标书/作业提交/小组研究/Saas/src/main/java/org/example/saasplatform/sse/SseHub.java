package org.example.saasplatform.sse;

import com.alibaba.fastjson2.JSON;
import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import org.example.saasplatform.entity.AuditTaskEvent;
import org.example.saasplatform.mapper.AuditTaskEventMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.time.LocalDateTime;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;

/**
 * Central SSE hub — manages SseEmitter connections per task.
 * <p>
 * - Subscribe: register new emitter for a taskId, optionally replay missed events
 * - SendEvent: persist to DB + push to all active emitters
 * - Cleanup: auto-remove on completion/timeout/error
 */
@Component
public class SseHub {

    private static final Logger log = LoggerFactory.getLogger(SseHub.class);

    // taskId → list of active emitters (thread-safe)
    private final ConcurrentHashMap<Long, List<SseEmitter>> emitters = new ConcurrentHashMap<>();

    @Autowired
    private AuditTaskEventMapper eventMapper;

    /**
     * Subscribe to SSE stream for a task.
     * @param taskId the audit task to subscribe to
     * @param lastEventId optional: replay events after this event ID (for reconnect)
     * @return SseEmitter (never times out)
     */
    public SseEmitter subscribe(Long taskId, String lastEventId) {
        SseEmitter emitter = new SseEmitter(0L); // no timeout

        emitters.computeIfAbsent(taskId, k -> new CopyOnWriteArrayList<>()).add(emitter);

        emitter.onCompletion(() -> removeEmitter(taskId, emitter, "completed"));
        emitter.onTimeout(() -> removeEmitter(taskId, emitter, "timeout"));
        emitter.onError(e -> removeEmitter(taskId, emitter, "error: " + e.getMessage()));

        // Send initial heartbeat
        try {
            emitter.send(SseEmitter.event()
                    .name("connected")
                    .data("{\"taskId\":" + taskId + "}"));
        } catch (IOException e) {
            removeEmitter(taskId, emitter, "initial send failed");
            return emitter;
        }

        // Replay missed events if reconnecting
        if (lastEventId != null && !lastEventId.isEmpty()) {
            try {
                replayEvents(taskId, Long.parseLong(lastEventId), emitter);
            } catch (NumberFormatException e) {
                log.warn("Invalid Last-Event-ID: {}", lastEventId);
            }
        }

        log.info("SSE subscriber added: taskId={}, totalSubscribers={}",
                taskId, emitters.getOrDefault(taskId, List.of()).size());
        return emitter;
    }

    /**
     * Send an event: persist to DB async, then push to all active emitters.
     */
    public void sendEvent(Long taskId, String eventName, Object data) {
        // 1. Persist to DB asynchronously (fire-and-forget — SSE push is not blocked by DB)
        AuditTaskEvent event = new AuditTaskEvent();
        event.setTaskId(taskId);
        event.setEventType(eventName);
        event.setEventData(JSON.toJSONString(data));
        event.setCreatedAt(LocalDateTime.now());
        CompletableFuture.runAsync(() -> eventMapper.insert(event));

        // 2. Push to all active emitters
        List<SseEmitter> taskEmitters = emitters.get(taskId);
        if (taskEmitters == null || taskEmitters.isEmpty()) {
            log.debug("No active emitters for taskId={}, event persisted only", taskId);
            return;
        }

        for (SseEmitter emitter : taskEmitters) {
            try {
                emitter.send(SseEmitter.event()
                        .id(event.getId().toString())
                        .name(eventName)
                        .data(data));
            } catch (IOException e) {
                removeEmitter(taskId, emitter, "send failed: " + e.getMessage());
            }
        }
    }

    // ── Replay ──────────────────────────────────────────────

    private void replayEvents(Long taskId, Long afterEventId, SseEmitter emitter) {
        List<AuditTaskEvent> events = eventMapper.selectList(
                new LambdaQueryWrapper<AuditTaskEvent>()
                        .eq(AuditTaskEvent::getTaskId, taskId)
                        .gt(AuditTaskEvent::getId, afterEventId)
                        .orderByAsc(AuditTaskEvent::getId));

        for (AuditTaskEvent event : events) {
            try {
                emitter.send(SseEmitter.event()
                        .id(event.getId().toString())
                        .name(event.getEventType())
                        .data(JSON.parse(event.getEventData())));
            } catch (IOException e) {
                log.warn("Replay interrupted for taskId={}: {}", taskId, e.getMessage());
                break;
            }
        }
        log.info("Replayed {} events for taskId={} after eventId={}", events.size(), taskId, afterEventId);
    }

    // ── Cleanup ─────────────────────────────────────────────

    private void removeEmitter(Long taskId, SseEmitter emitter, String reason) {
        List<SseEmitter> list = emitters.get(taskId);
        if (list != null) {
            list.remove(emitter);
            if (list.isEmpty()) {
                emitters.remove(taskId);
            }
        }
        log.debug("SSE emitter removed: taskId={}, reason={}, remaining={}",
                taskId, reason, list.size());
    }
}
