package org.example.saasplatform.service;

import org.example.saasplatform.entity.AuditTask;

public interface AuditEngineService {

    /**
     * Execute the full 4-stage audit lifecycle for a task.
     * <p>
     * Stages:
     * 1. PENDING → PROCESSING (DB update + SSE "progress")
     * 2. Progress updates (SSE "progress" events at 30%, 60%)
     * 3. Call RustApiClient (mock) for analysis
     * 4. COMPLETED (DB update + SSE "finding" + "complete")
     * <p>
     * On error: FAILED (DB update + SSE "error")
     */
    void executeAudit(AuditTask task);
}
