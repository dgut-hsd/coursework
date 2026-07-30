package org.example.saasplatform.mapper;

import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.example.saasplatform.entity.AuditTaskEvent;

@Mapper
public interface AuditTaskEventMapper extends BaseMapper<AuditTaskEvent> {
    // NOT tenant-filtered — queries by task_id directly
}
