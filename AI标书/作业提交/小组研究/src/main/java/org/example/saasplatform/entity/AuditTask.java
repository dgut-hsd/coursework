package org.example.saasplatform.entity;

import com.baomidou.mybatisplus.annotation.*;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.example.saasplatform.handler.StringListJsonTypeHandler;

import java.time.LocalDateTime;
import java.util.List;

@Data
@NoArgsConstructor
@AllArgsConstructor
@TableName(value = "audit_task", autoResultMap = true)
public class AuditTask {

    // ── Status constants ──────────────────────────────
    public static final String STATUS_PENDING = "PENDING";
    public static final String STATUS_PROCESSING = "PROCESSING";
    public static final String STATUS_COMPLETED = "COMPLETED";
    public static final String STATUS_FAILED = "FAILED";

    @TableId(type = IdType.AUTO)
    private Long id;
    private Long tenantId;
    private Long projectId;
    private String status;

    @TableField(typeHandler = StringListJsonTypeHandler.class)
    private List<String> enabledChecks;

    @Version
    private Integer version;
    private LocalDateTime createdAt;
}
