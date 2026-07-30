package org.example.saasplatform.dispatch;

import org.example.saasplatform.entity.AuditTask;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.data.redis.core.StringRedisTemplate;
import org.springframework.stereotype.Service;

/**
 * Redis List dispatcher — LPUSH taskId to a Redis list.
 * A companion worker BRPOPs from the list.
 * Activated when {@code audit.dispatcher.type=redis-list}.
 */
@Service
@ConditionalOnProperty(name = "audit.dispatcher.type", havingValue = "redis-list")
public class RedisListAuditTaskDispatcher implements AuditTaskDispatcher {

    private static final Logger log = LoggerFactory.getLogger(RedisListAuditTaskDispatcher.class);
    private static final String QUEUE_KEY = "audit:task:list";

    @Autowired
    private StringRedisTemplate redisTemplate;

    @Override
    public void dispatch(AuditTask task) {
        redisTemplate.opsForList().leftPush(QUEUE_KEY, task.getId().toString());
        log.info("Redis List push: taskId={}, queueSize={}",
                task.getId(), redisTemplate.opsForList().size(QUEUE_KEY));
    }
}
