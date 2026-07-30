package org.example.saasplatform.controller;

import com.baomidou.mybatisplus.extension.plugins.pagination.Page;
import org.example.saasplatform.common.Result;
import org.example.saasplatform.dto.CreateProjectRequest;
import org.example.saasplatform.entity.Project;
import org.example.saasplatform.service.ProjectService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/projects")
public class ProjectController {

    @Autowired
    private ProjectService projectService;

    @PostMapping
    public Result<Project> create(@RequestBody CreateProjectRequest request) {
        Project project = new Project();
        project.setName(request.getName());
        project.setDescription(request.getDescription());
        return Result.success(projectService.create(project));
    }

    @GetMapping("/page")
    public Result<Page<Project>> page(
            @RequestParam(defaultValue = "1") int pageNum,
            @RequestParam(defaultValue = "10") int pageSize,
            @RequestParam(required = false) String name) {
        return Result.success(projectService.page(pageNum, pageSize, name));
    }

    @GetMapping("/{id}")
    public Result<Project> getById(@PathVariable Long id) {
        return Result.success(projectService.getById(id));
    }

    @PutMapping("/{id}")
    public Result<Project> update(@PathVariable Long id, @RequestBody CreateProjectRequest request) {
        Project project = new Project();
        project.setId(id);
        project.setName(request.getName());
        project.setDescription(request.getDescription());
        return Result.success(projectService.update(project));
    }

    @DeleteMapping("/{id}")
    public Result<Void> delete(@PathVariable Long id) {
        projectService.delete(id);
        return Result.success();
    }
}
