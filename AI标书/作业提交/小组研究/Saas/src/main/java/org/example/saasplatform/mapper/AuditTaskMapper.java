package org.example.saasplatform.mapper;

import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.example.saasplatform.entity.AuditTask;

@Mapper
public interface AuditTaskMapper extends BaseMapper<AuditTask> {
    // Inherits 17 built-in CRUD methods + tenant filtering via TenantLineInnerInterceptor
}
