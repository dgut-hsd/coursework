package org.example.saasplatform.service.impl;

import com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper;
import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.common.BaseContext;
import org.example.saasplatform.entity.Project;
import org.example.saasplatform.mapper.ProjectMapper;
import org.example.saasplatform.service.ProjectService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import java.time.LocalDateTime;

@Service
public class ProjectServiceImpl implements ProjectService {

    @Autowired
    private ProjectMapper projectMapper;

    @Override
    public Project create(Project project) {
        project.setTenantId(BaseContext.getCurrentTenantId());
        project.setCreatedAt(LocalDateTime.now());
        projectMapper.insert(project);
        return project;
    }

    @Override
    public Page<Project> page(int pageNum, int pageSize, String name) {
        Page<Project> page = new Page<>(pageNum, pageSize);
        LambdaQueryWrapper<Project> wrapper = new LambdaQueryWrapper<>();
        if (name != null && !name.isBlank()) {
            wrapper.like(Project::getName, name);
        }
        wrapper.orderByDesc(Project::getCreatedAt);
        // TenantLineInnerInterceptor auto-injects WHERE tenant_id = ?
        return projectMapper.selectPage(page, wrapper);
    }

    @Override
    public Project getById(Long id) {
        // TenantLine interceptor auto-adds tenant_id filter
        return projectMapper.selectById(id);
    }

    @Override
    public Project update(Project project) {
        // Guard: ensure tenant ownership
        project.setTenantId(BaseContext.getCurrentTenantId());
        projectMapper.updateById(project);
        return projectMapper.selectById(project.getId());
    }

    @Override
    public void delete(Long id) {
        // TenantLine interceptor auto-adds tenant_id filter
        projectMapper.deleteById(id);
    }
}
