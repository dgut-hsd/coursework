package org.example.saasplatform.dto;

import java.util.List;

public class CreateTaskRequest {
    private Long projectId;
    private List<String> enabledChecks;

    public Long getProjectId() { return projectId; }
    public void setProjectId(Long projectId) { this.projectId = projectId; }

    public List<String> getEnabledChecks() { return enabledChecks; }
    public void setEnabledChecks(List<String> enabledChecks) { this.enabledChecks = enabledChecks; }
}
