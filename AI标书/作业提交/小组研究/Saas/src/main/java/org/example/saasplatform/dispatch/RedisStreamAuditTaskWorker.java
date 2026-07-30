package org.example.saasplatform.dispatch;

import jakarta.annotation.PostConstruct;
import org.example.saasplatform.entity.AuditTask;
import org.example.saasplatform.mapper.AuditTaskMapper;
import org.example.saasplatform.service.AuditEngineService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.data.domain.Range;
import org.springframework.data.redis.connection.stream.Consumer;
import org.springframework.data.redis.connection.stream.MapRecord;
import org.springframework.data.redis.connection.stream.ReadOffset;
import org.springframework.data.redis.connection.stream.StreamOffset;
import org.springframework.data.redis.core.StringRedisTemplate;
import org.springframework.stereotype.Component;

import java.time.Duration;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * Redis Streams consumer worker with Consumer Group and Dead Letter Queue.
 * <p>
 * - XREADGROUP reads pending messages
 * - On success: XACK
 * - On failure: XADD to DLQ (dead letter queue), then XACK
 * <p>
 * Activated when {@code audit.dispatcher.type=redis-stream}.
 */
@Component
@ConditionalOnProperty(name = "audit.dispatcher.type", havingValue = "redis-stream")
public class RedisStreamAuditTaskWorker {

    private static final Logger log = LoggerFactory.getLogger(RedisStreamAuditTaskWorker.class);
    private static final String STREAM_KEY = "audit:task:stream";
    private static final String DLQ_KEY = "audit:task:dlq";
    private static final String GROUP = "audit-workers";
    private static final String CONSUMER = "worker-" + UUID.randomUUID().toString().substring(0, 8);

    @Autowired
    private StringRedisTemplate redisTemplate;

    @Autowired
    private AuditEngineService auditEngineService;

    @Autowired
    private AuditTaskMapper auditTaskMapper;

    private volatile boolean running = true;

    @PostConstruct
    public void start() {
        new Thread(this::consumeLoop, "audit-stream-worker").start();
        log.info("Redis Stream worker started: consumer={}", CONSUMER);
    }

    @SuppressWarnings("unchecked")
    private void consumeLoop() {
        while (running) {
            try {
                List<MapRecord<String, Object, Object>> records = redisTemplate.opsForStream()
                        .read(
                                Consumer.from(GROUP, CONSUMER),
                                org.springframework.data.redis.connection.stream.StreamReadOptions.empty()
                                        .count(1)
                                        .block(Duration.ofSeconds(5)),
                                StreamOffset.create(STREAM_KEY, ReadOffset.lastConsumed())
                        );

                if (records == null || records.isEmpty()) {
                    continue;
                }

                for (MapRecord<String, Object, Object> record : records) {
                    processRecord(record);
                }
            } catch (Exception e) {
                log.error("Consumer loop error", e);
                try {
                    Thread.sleep(1000);
                } catch (InterruptedException ignored) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }
    }

    private void processRecord(MapRecord<String, Object, Object> record) {
        try {
            Long taskId = Long.valueOf(record.getValue().get("taskId").toString());

            // Fetch full task from DB
            AuditTask task = auditTaskMapper.selectById(taskId);
            if (task == null) {
                log.warn("Task not found in DB: taskId={}, acking anyway", taskId);
                redisTemplate.opsForStream().acknowledge(STREAM_KEY, GROUP, record.getId().getValue());
                return;
            }

            // Execute audit
            auditEngineService.executeAudit(task);

            // ACK on success
            redisTemplate.opsForStream().acknowledge(STREAM_KEY, GROUP, record.getId().getValue());
            log.info("Task completed and acked: taskId={}", taskId);

        } catch (Exception e) {
            log.error("Task processing failed: recordId={}", record.getId(), e);
            // Move to DLQ
            moveToDlq(record, e.getMessage());
        }
    }

    private void moveToDlq(MapRecord<String, Object, Object> record, String error) {
        Map<String, String> dlqEntry = new HashMap<>();
        record.getValue().forEach((k, v) -> dlqEntry.put(k.toString(), v.toString()));
        dlqEntry.put("error", error);
        dlqEntry.put("original_stream", STREAM_KEY);
        redisTemplate.opsForStream().add(DLQ_KEY, dlqEntry);
        // ACK the original so it's not re-delivered
        redisTemplate.opsForStream().acknowledge(STREAM_KEY, GROUP, record.getId().getValue());
        log.warn("Moved to DLQ: recordId={}, error={}", record.getId(), error);
    }
}
