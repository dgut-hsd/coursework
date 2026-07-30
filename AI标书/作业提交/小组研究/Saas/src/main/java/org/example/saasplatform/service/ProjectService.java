package org.example.saasplatform.service;

import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.entity.Project;

public interface ProjectService {

    Project create(Project project);

    Page<Project> page(int pageNum, int pageSize, String name);

    Project getById(Long id);

    Project update(Project project);

    void delete(Long id);
}
