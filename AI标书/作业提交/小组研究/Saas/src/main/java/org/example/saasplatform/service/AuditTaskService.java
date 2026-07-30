package org.example.saasplatform.service;

import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.entity.AuditTask;

public interface AuditTaskService {

    /**
     * Create audit task — INSERT PENDING, dispatch after transaction commit.
     */
    AuditTask createTask(AuditTask task);

    /**
     * Paginated query (tenant-isolated, dynamic filters).
     */
    Page<AuditTask> pageTasks(int pageNum, int pageSize, String status, String startTime);

    /**
     * Get task detail (tenant-isolated).
     */
    AuditTask getTaskById(Long id);

    /**
     * Update task status (for engine/callback use).
     */
    void updateTaskStatus(Long taskId, String status);
}
