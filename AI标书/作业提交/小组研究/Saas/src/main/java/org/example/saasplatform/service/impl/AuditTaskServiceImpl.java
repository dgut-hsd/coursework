package org.example.saasplatform.service.impl;

import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.common.BaseContext;
import org.example.saasplatform.entity.AuditTask;
import org.example.saasplatform.mapper.AuditTaskMapper;
import org.example.saasplatform.service.AuditTaskService;
import org.example.saasplatform.dispatch.AuditTaskDispatcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.transaction.support.TransactionSynchronization;
import org.springframework.transaction.support.TransactionSynchronizationManager;

import java.time.LocalDateTime;

@Service
public class AuditTaskServiceImpl implements AuditTaskService {

    private static final Logger log = LoggerFactory.getLogger(AuditTaskServiceImpl.class);

    @Autowired
    private AuditTaskMapper auditTaskMapper;

    @Autowired
    private AuditTaskDispatcher dispatcher;

    @Override
    @Transactional
    public AuditTask createTask(AuditTask task) {
        // Set context values
        task.setTenantId(BaseContext.getCurrentTenantId());
        task.setStatus(AuditTask.STATUS_PENDING);
        task.setVersion(1);
        task.setCreatedAt(LocalDateTime.now());

        // Insert to DB
        auditTaskMapper.insert(task);

        // Dispatch ONLY after DB transaction commits
        // Prevents race condition: worker reads task before INSERT is committed
        TransactionSynchronizationManager.registerSynchronization(
                new TransactionSynchronization() {
                    @Override
                    public void afterCommit() {
                        log.info("Dispatching task after commit: taskId={}", task.getId());
                        dispatcher.dispatch(task);
                    }
                });

        return task;
    }

    @Override
    public Page<AuditTask> pageTasks(int pageNum, int pageSize, String status, String startTime) {
        Page<AuditTask> page = new Page<>(pageNum, pageSize);
        LambdaQueryWrapper<AuditTask> wrapper = new LambdaQueryWrapper<>();
        if (status != null && !status.isBlank()) {
            wrapper.eq(AuditTask::getStatus, status);
        }
        if (startTime != null && !startTime.isBlank()) {
            wrapper.ge(AuditTask::getCreatedAt, startTime);
        }
        wrapper.orderByDesc(AuditTask::getCreatedAt);
        // TenantLineInnerInterceptor auto-injects WHERE tenant_id = ?
        return auditTaskMapper.selectPage(page, wrapper);
    }

    @Override
    public AuditTask getTaskById(Long id) {
        return auditTaskMapper.selectById(id);
    }

    @Override
    public void updateTaskStatus(Long taskId, String status) {
        AuditTask task = new AuditTask();
        task.setId(taskId);
        task.setStatus(status);
        auditTaskMapper.updateById(task);
    }
}
