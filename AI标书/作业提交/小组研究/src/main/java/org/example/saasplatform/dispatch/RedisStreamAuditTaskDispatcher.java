package org.example.saasplatform.dispatch;

import jakarta.annotation.PostConstruct;
import org.example.saasplatform.entity.AuditTask;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.data.redis.core.StringRedisTemplate;
import org.springframework.stereotype.Service;

import java.util.Map;

/**
 * Redis Streams dispatcher — XADD task to a stream with Consumer Group.
 * Activated when {@code audit.dispatcher.type=redis-stream}.
 */
@Service
@ConditionalOnProperty(name = "audit.dispatcher.type", havingValue = "redis-stream")
public class RedisStreamAuditTaskDispatcher implements AuditTaskDispatcher {

    private static final Logger log = LoggerFactory.getLogger(RedisStreamAuditTaskDispatcher.class);
    private static final String STREAM_KEY = "audit:task:stream";
    private static final String GROUP = "audit-workers";

    @Autowired
    private StringRedisTemplate redisTemplate;

    @PostConstruct
    public void init() {
        try {
            redisTemplate.opsForStream().createGroup(STREAM_KEY, GROUP);
            log.info("Consumer group '{}' created for stream '{}'", GROUP, STREAM_KEY);
        } catch (Exception e) {
            // Group may already exist — that's fine
            log.debug("Consumer group already exists: {}", e.getMessage());
        }
    }

    @Override
    public void dispatch(AuditTask task) {
        Map<String, String> fields = Map.of(
                "taskId", task.getId().toString(),
                "tenantId", task.getTenantId().toString()
        );
        var recordId = redisTemplate.opsForStream().add(STREAM_KEY, fields);
        log.info("Redis Stream XADD: taskId={}, recordId={}", task.getId(), recordId);
    }
}
