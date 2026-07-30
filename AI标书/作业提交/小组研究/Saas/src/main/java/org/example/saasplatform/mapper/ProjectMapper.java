package org.example.saasplatform.mapper;

import com.baomidou.mybatisplus.core.mapper.BaseMapper;
import org.apache.ibatis.annotations.Mapper;
import org.example.saasplatform.entity.Project;

@Mapper
public interface ProjectMapper extends BaseMapper<Project> {
    // Tenant filtering is automatic via TenantLineInnerInterceptor
}
